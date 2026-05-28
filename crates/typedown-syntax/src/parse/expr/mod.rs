//! Parser method for parsing many kinds of expressions

use typedown_types::{diagnostic::Diagnostic, stream::Utf8Stream, syntax_kind::SyntaxKind};

use super::constants::*;
use crate::{
  green::GreenNode,
  lex::ctx::LexMode,
  parse::ctx::{ParseCtx, expr_ctx::ExprCtx},
};

impl<S: Utf8Stream> ParseCtx<S> {
  /// General expression, including formula and yaml.
  pub(in crate::parse) fn parse_expr(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mode = self.lex_ctx.mode();
    let peek = self.lex_ctx.peek(SKIP_WS | SKIP_COMMENT, mode);

    match peek.token.kind() {
      SyntaxKind::Newline => {
        let peek_after = self
          .lex_ctx
          .peek(SKIP_NEWLINE | SKIP_WS | SKIP_COMMENT, mode);
        if peek_after.token.kind() == SyntaxKind::YamlIndent {
          self.parse_block_seq_or_mapping()
        } else {
          // No indent after newline: parse as formula (will likely produce an error)
          self.parse_formula_expr()
        }
      }
      // `|`, `>`, or `-` at the start
      SyntaxKind::YamlOp => {
        let text: String = peek.token.text().collect();
        match text.as_str() {
          "|" => self.parse_literal_block_str_lit(),
          ">" => self.parse_folded_block_str_lit(),
          "-" => {
            let after = self.lex_ctx.peek_yaml_nth(1, SKIP_NONE);
            if after.token.kind() == SyntaxKind::Whitespace {
              self.parse_block_seq_lit()
            } else {
              self.parse_formula_expr()
            }
          }
          _ => self.parse_formula_expr(),
        }
      }
      // Ident followed by colon: inline mapping
      // We need to handle this specially as in the following case:
      // -  key: value
      // #^^
      // # this is not an indent
      //    key2: value2
      // #^^
      // # this is an indent
      SyntaxKind::Ident if mode == LexMode::YamlFrontmatter => {
        let after_ident = self.lex_ctx.peek_yaml_nth(1, SKIP_WS | SKIP_COMMENT);
        if after_ident.token.kind() == SyntaxKind::Colon {
          self.parse_inline_block_mapping_lit()
        } else {
          self.parse_formula_expr()
        }
      }
      // Everything else: formula expression
      _ => self.parse_formula_expr(),
    }
  }

  fn expr_skip_flags(&self) -> u16 {
    let mut skip = SKIP_WS | SKIP_COMMENT;
    if self.expr_ctx_stack.should_expr_span_newline() {
      skip |= SKIP_NEWLINE;
    }
    skip
  }

  /// Formula expressions: Pratt-parsed expressions that follow most programming language rules
  pub(in crate::parse) fn parse_formula_expr(&mut self) -> (GreenNode, Option<ExprCtx>) {
    self.pratt_parse_expr(0)
  }

  fn pratt_parse_expr(&mut self, min_bp: u8) -> (GreenNode, Option<ExprCtx>) {
    let mode = self.lex_ctx.mode();

    // Handle prefix operators
    let peek = self.lex_ctx.peek(self.expr_skip_flags(), mode);
    let (mut lhs, early_exit) = if peek.token.kind() == SyntaxKind::YamlOp {
      let op_text: String = peek.token.text().collect();
      if let Some(((), right_bp)) = prefix_binding_power(&op_text) {
        let mut children = vec![];
        // Consume the prefix operator
        self.advance(&mut children, self.expr_skip_flags(), mode);
        // Parse operand with the prefix's right binding power
        let (operand, exit) = self.pratt_parse_expr(right_bp);
        children.push(operand);
        (self.emit(SyntaxKind::UnaryExpr, &children), exit)
      } else {
        // Not a prefix op, parse as primary
        self.parse_primary_expr()
      }
    } else {
      self.parse_primary_expr()
    };

    if early_exit.is_some() {
      return (lhs, early_exit);
    }

    // Infix/postfix loop
    loop {
      let peek = self.lex_ctx.peek(self.expr_skip_flags(), mode);

      // Check for call expression: ident followed by `(`
      if peek.token.kind() == SyntaxKind::LParen {
        let (node, exit) = self.parse_call_expr(lhs);
        lhs = node;
        if exit.is_some() {
          return (lhs, exit);
        }
        continue;
      }

      // Check for infix operator
      if peek.token.kind() != SyntaxKind::YamlOp {
        break;
      }

      let op_text: String = peek.token.text().collect();

      // Check postfix first
      if let Some((left_bp, ())) = postfix_binding_power(&op_text) {
        if left_bp < min_bp {
          break;
        }
        let mut children = vec![lhs];
        self.advance(&mut children, self.expr_skip_flags(), mode);
        lhs = self.emit(SyntaxKind::UnaryExpr, &children);
        continue;
      }

      // Check infix
      if let Some((left_bp, right_bp)) = infix_binding_power(&op_text) {
        if left_bp < min_bp {
          break;
        }
        let mut children = vec![lhs];
        // Consume the infix operator
        self.advance(&mut children, self.expr_skip_flags(), mode);
        // Parse right-hand side
        let (rhs, exit) = self.pratt_parse_expr(right_bp);
        children.push(rhs);
        lhs = self.emit(SyntaxKind::BinaryExpr, &children);
        if exit.is_some() {
          return (lhs, exit);
        }
        continue;
      }

      // Not a recognized operator, stop
      break;
    }

    (lhs, None)
  }

  /// Parse a call expression: `callee(arg1, arg2, ...)`.
  /// `callee` has already been parsed and is passed in.
  fn parse_call_expr(&mut self, callee: GreenNode) -> (GreenNode, Option<ExprCtx>) {
    let mode = self.lex_ctx.mode();
    debug_assert!(
      self.lex_ctx.peek(self.expr_skip_flags(), mode).token.kind() == SyntaxKind::LParen,
      "[ParseCtx::parse_call_expr] Expected next token to be LParen"
    );

    let mut children = vec![callee];
    self.expr_ctx_stack.enter(ExprCtx::Call);

    // Consume `(`
    self.advance(&mut children, self.expr_skip_flags(), mode);

    // Check for empty args `()`
    let peek = self.lex_ctx.peek(SKIP_ALL_TRIVIA, mode);
    if peek.token.kind() == SyntaxKind::RParen {
      self.advance(&mut children, SKIP_ALL_TRIVIA, mode);
      self.expr_ctx_stack.exit(ExprCtx::Call);
      return (self.emit(SyntaxKind::CallExpr, &children), None);
    }

    // Parse first argument
    let (arg, early_exit) = self.parse_expr();
    children.push(arg);
    if early_exit.is_some_and(|ctx| ctx != ExprCtx::Call) {
      self.expr_ctx_stack.exit(ExprCtx::Call);
      return (self.emit(SyntaxKind::CallExpr, &children), early_exit);
    }

    // Parse remaining arguments
    loop {
      let peek = self.lex_ctx.peek(SKIP_ALL_TRIVIA, mode);
      match peek.token.kind() {
        SyntaxKind::RParen => {
          self.advance(&mut children, SKIP_ALL_TRIVIA, mode);
          break;
        }
        SyntaxKind::Comma => {
          self.advance(&mut children, SKIP_ALL_TRIVIA, mode);

          // Trailing comma before `)` is allowed
          let peek = self.lex_ctx.peek(SKIP_ALL_TRIVIA, mode);
          if peek.token.kind() == SyntaxKind::RParen {
            self.advance(&mut children, SKIP_ALL_TRIVIA, mode);
            break;
          }

          let (arg, early_exit) = self.parse_expr();
          children.push(arg);
          if early_exit.is_some_and(|ctx| ctx != ExprCtx::Call) {
            self.expr_ctx_stack.exit(ExprCtx::Call);
            return (self.emit(SyntaxKind::CallExpr, &children), early_exit);
          }
        }
        SyntaxKind::Eof => {
          self.diagnostics.push(Diagnostic::MissingSyntaxNode {
            expected: SyntaxKind::RParen,
            start_offset: self.offset(),
            end_offset: self.offset(),
          });
          break;
        }
        _ => {
          let handler = self.expr_ctx_stack.find_handler(&peek.token);
          if handler.is_some_and(|ctx| ctx != ExprCtx::Call) {
            self.expr_ctx_stack.exit(ExprCtx::Call);
            return (self.emit(SyntaxKind::CallExpr, &children), handler);
          }
          if let Some(ctx) = self.synchronize_call_expr(&mut children) {
            self.expr_ctx_stack.exit(ExprCtx::Call);
            return (self.emit(SyntaxKind::CallExpr, &children), Some(ctx));
          }
        }
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::Call);
    (self.emit(SyntaxKind::CallExpr, &children), None)
  }

  // Stop on Comma and RParen
  fn synchronize_call_expr(&mut self, children: &mut Vec<GreenNode>) -> Option<ExprCtx> {
    let mut error_children = vec![];
    let result = loop {
      let peek = self.lex_ctx.peek(SKIP_NONE, self.lex_ctx.mode());
      match peek.token.kind() {
        SyntaxKind::Comma | SyntaxKind::RParen | SyntaxKind::Eof => break None,
        _ => {
          if let Some(ctx) = self.consume_or_delegate(ExprCtx::Call, &mut error_children) {
            break Some(ctx);
          }
        }
      }
    };
    if !error_children.is_empty() {
      children.push(self.emit(SyntaxKind::Error, &error_children));
    }
    result
  }

  /// Parse a primary expression (an operand): literal, ident, paren, etc.
  pub(in crate::parse) fn parse_primary_expr(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mode = self.lex_ctx.mode();
    let peek = self.lex_ctx.peek(self.expr_skip_flags(), mode);

    match peek.token.kind() {
      SyntaxKind::Number => self.parse_number_lit(),
      SyntaxKind::DqStrStart => self.parse_dq_str_lit(),
      SyntaxKind::SqStrStart => self.parse_sq_str_lit(),
      SyntaxKind::InlineCode | SyntaxKind::CodeBlock => self.parse_code_lit(),
      SyntaxKind::InlineMath | SyntaxKind::MathBlock => self.parse_math_lit(),
      SyntaxKind::Ident => self.parse_ident_lit(),
      SyntaxKind::LParen => self.parse_paren_expr(),
      SyntaxKind::LBracket => self.parse_list_lit(),
      SyntaxKind::LBrace => self.parse_dict_lit(),
      _ => {
        // Check if an outer context can handle this token
        let handler = self.expr_ctx_stack.find_handler(&peek.token);
        if handler.is_some() {
          // Don't consume: let the caller handle it
          self.diagnostics.push(Diagnostic::MissingSyntaxNode {
            expected: SyntaxKind::PrimaryExpr,
            start_offset: self.offset(),
            end_offset: self.offset(),
          });
          (self.emit(SyntaxKind::PrimaryExpr, &[]), handler)
        } else {
          // No one can handle it: consume as error
          let mut children = vec![];
          self.advance(&mut children, self.expr_skip_flags(), mode);
          let bad = children.pop().unwrap();
          children.push(self.emit(SyntaxKind::Error, &[bad]));
          self.diagnostics.push(Diagnostic::MissingSyntaxNode {
            expected: SyntaxKind::PrimaryExpr,
            start_offset: self.offset(),
            end_offset: self.offset(),
          });
          (self.emit(SyntaxKind::PrimaryExpr, &children), None)
        }
      }
    }
  }

  /// Parse a parenthesized expression: `(expr)`.
  pub(in crate::parse) fn parse_paren_expr(&mut self) -> (GreenNode, Option<ExprCtx>) {
    debug_assert!(
      self
        .lex_ctx
        .peek(SKIP_ALL_TRIVIA, self.lex_ctx.mode())
        .token
        .kind()
        == SyntaxKind::LParen,
      "[ParseCtx::parse_paren_expr] Expected next token to be LParen"
    );
    let mut children = vec![];
    self.expr_ctx_stack.enter(ExprCtx::Paren);

    // Consume `(`
    let offset = self.offset();
    self.consume(
      &mut children,
      SKIP_ALL_TRIVIA,
      self.lex_ctx.mode(),
      SyntaxKind::LParen,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::LParen,
        start_offset: offset,
        end_offset: self.offset(),
      },
    );

    // Parse inner expression
    let (inner, early_exit) = self.parse_formula_expr();
    children.push(inner);

    if early_exit.is_some_and(|ctx| ctx != ExprCtx::Paren) {
      self.expr_ctx_stack.exit(ExprCtx::Paren);
      return (self.emit(SyntaxKind::ParenExpr, &children), early_exit);
    }

    // Consume `)`
    let offset = self.offset();
    self.consume(
      &mut children,
      SKIP_ALL_TRIVIA,
      self.lex_ctx.mode(),
      SyntaxKind::RParen,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::RParen,
        start_offset: offset,
        end_offset: self.offset(),
      },
    );

    self.expr_ctx_stack.exit(ExprCtx::Paren);
    (self.emit(SyntaxKind::ParenExpr, &children), None)
  }

  /// Parse a flow list literal: `[expr, expr, ...]`.
  pub(in crate::parse) fn parse_list_lit(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let outer_skip = self.expr_skip_flags()
      | if self.expr_ctx_stack.should_expr_skip_indent() {
        SKIP_INDENT_DEDENT
      } else {
        0
      };
    debug_assert!(
      self
        .lex_ctx
        .peek(outer_skip, self.lex_ctx.mode())
        .token
        .kind()
        == SyntaxKind::LBracket,
      "[ParseCtx::parse_list_lit] Expected next token to be LBracket"
    );

    let mode = self.lex_ctx.mode();
    let mut children = vec![];
    self.expr_ctx_stack.enter(ExprCtx::List);

    // Consume `[`
    let offset = self.offset();
    self.consume(
      &mut children,
      outer_skip,
      mode,
      SyntaxKind::LBracket,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::LBracket,
        start_offset: offset,
        end_offset: self.offset(),
      },
    );

    // Check for empty list `[]`
    let peek = self.lex_ctx.peek(SKIP_ALL_TRIVIA, mode);
    if peek.token.kind() == SyntaxKind::RBracket {
      self.advance(&mut children, SKIP_ALL_TRIVIA, mode);
      self.expr_ctx_stack.exit(ExprCtx::List);
      return (self.emit(SyntaxKind::ListLit, &children), None);
    }

    // Parse first item (no leading comma)
    let (item, early_exit) = self.parse_expr();
    children.push(item);
    if early_exit.is_some_and(|ctx| ctx != ExprCtx::List) {
      self.expr_ctx_stack.exit(ExprCtx::List);
      return (self.emit(SyntaxKind::ListLit, &children), early_exit);
    }

    // Parse remaining items
    loop {
      let peek = self.lex_ctx.peek(SKIP_ALL_TRIVIA, mode);
      match peek.token.kind() {
        // End of list
        SyntaxKind::RBracket => {
          self.advance(&mut children, SKIP_ALL_TRIVIA, mode);
          break;
        }
        // Separator: expect another item
        SyntaxKind::Comma => {
          self.advance(&mut children, SKIP_ALL_TRIVIA, mode);

          // Trailing comma before `]` is allowed
          let peek = self.lex_ctx.peek(SKIP_ALL_TRIVIA, mode);
          if peek.token.kind() == SyntaxKind::RBracket {
            self.advance(&mut children, SKIP_ALL_TRIVIA, mode);
            break;
          }

          let (item, early_exit) = self.parse_expr();
          children.push(item);
          if early_exit.is_some_and(|ctx| ctx != ExprCtx::List) {
            self.expr_ctx_stack.exit(ExprCtx::List);
            return (self.emit(SyntaxKind::ListLit, &children), early_exit);
          }
        }
        // EOF
        SyntaxKind::Eof => {
          self.diagnostics.push(Diagnostic::MissingSyntaxNode {
            expected: SyntaxKind::RBracket,
            start_offset: self.offset(),
            end_offset: self.offset(),
          });
          break;
        }
        // Unexpected token: check handler context
        _ => {
          let handler = self.expr_ctx_stack.find_handler(&peek.token);
          if handler.is_some_and(|ctx| ctx != ExprCtx::List) {
            // Outer context should handle this token
            self.expr_ctx_stack.exit(ExprCtx::List);
            return (self.emit(SyntaxKind::ListLit, &children), handler);
          }
          // Current context or no handler: synchronize
          if let Some(ctx) = self.synchronize_list_lit(&mut children) {
            self.expr_ctx_stack.exit(ExprCtx::List);
            return (self.emit(SyntaxKind::ListLit, &children), Some(ctx));
          }
        }
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::List);
    (self.emit(SyntaxKind::ListLit, &children), None)
  }

  // Stop on Comma and RBracket
  fn synchronize_list_lit(&mut self, children: &mut Vec<GreenNode>) -> Option<ExprCtx> {
    let mut error_children = vec![];
    let result = loop {
      let peek = self.lex_ctx.peek(SKIP_NONE, self.lex_ctx.mode());
      match peek.token.kind() {
        SyntaxKind::Comma | SyntaxKind::RBracket | SyntaxKind::Eof => break None,
        _ => {
          if let Some(ctx) = self.consume_or_delegate(ExprCtx::List, &mut error_children) {
            break Some(ctx);
          }
        }
      }
    };
    if !error_children.is_empty() {
      children.push(self.emit(SyntaxKind::Error, &error_children));
    }
    result
  }

  /// Parse a block expression (sequence or mapping) after a newline.
  pub(in crate::parse) fn parse_block_seq_or_mapping(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mode = self.lex_ctx.mode();
    let mut children = vec![];

    // Consume indent
    let offset = self.offset();
    self.consume(
      &mut children,
      SKIP_NEWLINE | SKIP_WS | SKIP_COMMENT,
      mode,
      SyntaxKind::YamlIndent,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::YamlIndent,
        start_offset: offset,
        end_offset: self.offset(),
      },
    );

    // Peek to decide: `-` means sequence, `ident` means mapping
    let peek = self
      .lex_ctx
      .peek(SKIP_NEWLINE | SKIP_WS | SKIP_COMMENT, mode);

    let (node, kind, early_exit) =
      if peek.token.kind() == SyntaxKind::YamlOp && peek.token.text().collect::<String>() == "-" {
        let (seq, early_exit) = self.parse_block_seq_lit();
        (seq, SyntaxKind::BlockSeqLit, early_exit)
      } else {
        let (mapping, early_exit) = self.parse_block_mapping_lit();
        (mapping, SyntaxKind::BlockMappingLit, early_exit)
      };
    children.push(node);

    // Consume the matching dedent
    let dedent_peek = self.lex_ctx.peek(SKIP_NEWLINE | SKIP_WS | SKIP_COMMENT, mode);
    if dedent_peek.token.kind() == SyntaxKind::YamlDedent {
      self.advance(&mut children, SKIP_NEWLINE | SKIP_WS | SKIP_COMMENT, mode);
    }

    (self.emit(kind, &children), early_exit)
  }

  /// Parse a block sequence literal: lines starting with `-`.
  pub(in crate::parse) fn parse_block_seq_lit(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mode = self.lex_ctx.mode();
    let mut children = vec![];
    self.expr_ctx_stack.enter(ExprCtx::BlockSeq);

    // Parse items
    loop {
      let peek = self
        .lex_ctx
        .peek(SKIP_NEWLINE | SKIP_WS | SKIP_COMMENT, mode);

      match peek.token.kind() {
        SyntaxKind::YamlDedent | SyntaxKind::Eof => break,
        SyntaxKind::YamlOp if peek.token.text().collect::<String>() == "-" => {
          let (item, early_exit) = self.parse_block_seq_item();
          children.push(item);
          if early_exit.is_some_and(|ctx| ctx != ExprCtx::BlockSeq) {
            self.expr_ctx_stack.exit(ExprCtx::BlockSeq);
            return (self.emit(SyntaxKind::BlockSeqLit, &children), early_exit);
          }
        }
        _ => {
          let handler = self.expr_ctx_stack.find_handler(&peek.token);
          if handler.is_some_and(|ctx| ctx != ExprCtx::BlockSeq) {
            self.expr_ctx_stack.exit(ExprCtx::BlockSeq);
            return (self.emit(SyntaxKind::BlockSeqLit, &children), handler);
          }
          if let Some(ctx) = self.synchronize_block_seq(&mut children) {
            self.expr_ctx_stack.exit(ExprCtx::BlockSeq);
            return (self.emit(SyntaxKind::BlockSeqLit, &children), Some(ctx));
          }
        }
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::BlockSeq);
    (self.emit(SyntaxKind::BlockSeqLit, &children), None)
  }

  /// Parse a single block sequence item: `- expr`.
  fn parse_block_seq_item(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mode = self.lex_ctx.mode();
    let mut children = vec![];

    // Consume `-`
    self.advance(&mut children, SKIP_NEWLINE | SKIP_WS | SKIP_COMMENT, mode);

    // Parse the value expression
    let (value, early_exit) = self.parse_expr();
    children.push(value);

    (self.emit(SyntaxKind::SequenceItem, &children), early_exit)
  }

  /// Parse a flow mapping literal: `{key: value, ...}`.
  pub(in crate::parse) fn parse_dict_lit(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let outer_skip = self.expr_skip_flags()
      | if self.expr_ctx_stack.should_expr_skip_indent() {
        SKIP_INDENT_DEDENT
      } else {
        0
      };
    debug_assert!(
      self
        .lex_ctx
        .peek(outer_skip, self.lex_ctx.mode())
        .token
        .kind()
        == SyntaxKind::LBrace,
      "[ParseCtx::parse_dict_lit] Expected next token to be LBrace"
    );

    let mode = self.lex_ctx.mode();
    let mut children = vec![];
    self.expr_ctx_stack.enter(ExprCtx::Dict);

    // Consume `{`
    let offset = self.offset();
    self.consume(
      &mut children,
      outer_skip,
      mode,
      SyntaxKind::LBrace,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::LBrace,
        start_offset: offset,
        end_offset: self.offset(),
      },
    );

    // Check for empty mapping `{}`
    let peek = self.lex_ctx.peek(SKIP_ALL_TRIVIA, mode);
    if peek.token.kind() == SyntaxKind::RBrace {
      self.advance(&mut children, SKIP_ALL_TRIVIA, mode);
      self.expr_ctx_stack.exit(ExprCtx::Dict);
      return (self.emit(SyntaxKind::DictLit, &children), None);
    }

    // Parse first entry
    let (entry, early_exit) = self.parse_dict_entry_lit();
    children.push(entry);
    if early_exit.is_some_and(|ctx| ctx != ExprCtx::Dict) {
      self.expr_ctx_stack.exit(ExprCtx::Dict);
      return (self.emit(SyntaxKind::DictLit, &children), early_exit);
    }

    // Parse remaining entries
    loop {
      let peek = self.lex_ctx.peek(SKIP_ALL_TRIVIA, mode);
      match peek.token.kind() {
        // End of mapping
        SyntaxKind::RBrace => {
          self.advance(&mut children, SKIP_ALL_TRIVIA, mode);
          break;
        }
        // Separator: expect another entry
        SyntaxKind::Comma => {
          self.advance(&mut children, SKIP_ALL_TRIVIA, mode);

          // Trailing comma before `}` is allowed
          let peek = self.lex_ctx.peek(SKIP_ALL_TRIVIA, mode);
          if peek.token.kind() == SyntaxKind::RBrace {
            self.advance(&mut children, SKIP_ALL_TRIVIA, mode);
            break;
          }

          let (entry, early_exit) = self.parse_dict_entry_lit();
          children.push(entry);
          if early_exit.is_some_and(|ctx| ctx != ExprCtx::Dict) {
            self.expr_ctx_stack.exit(ExprCtx::Dict);
            return (self.emit(SyntaxKind::DictLit, &children), early_exit);
          }
        }
        // EOF
        SyntaxKind::Eof => {
          self.diagnostics.push(Diagnostic::MissingSyntaxNode {
            expected: SyntaxKind::RBrace,
            start_offset: self.offset(),
            end_offset: self.offset(),
          });
          break;
        }
        // Unexpected token
        _ => {
          let handler = self.expr_ctx_stack.find_handler(&peek.token);
          if handler.is_some_and(|ctx| ctx != ExprCtx::Dict) {
            self.expr_ctx_stack.exit(ExprCtx::Dict);
            return (self.emit(SyntaxKind::DictLit, &children), handler);
          }
          if let Some(ctx) = self.synchronize_dict_lit(&mut children) {
            self.expr_ctx_stack.exit(ExprCtx::Dict);
            return (self.emit(SyntaxKind::DictLit, &children), Some(ctx));
          }
        }
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::Dict);
    (self.emit(SyntaxKind::DictLit, &children), None)
  }

  /// Parse a single mapping entry: `key: value`.
  fn parse_dict_entry_lit(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mode = self.lex_ctx.mode();
    let mut children = vec![];

    let peek = self.lex_ctx.peek(SKIP_ALL_TRIVIA, mode);

    // Missing key: `:` seen immediately
    if peek.token.kind() == SyntaxKind::Colon {
      self.diagnostics.push(Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::DictEntryKey,
        start_offset: self.offset(),
        end_offset: self.offset(),
      });
      // Emit empty MappingEntryKey as error
      children.push(self.emit(SyntaxKind::DictEntryKey, &[]));
    } else {
      // Key (required to be an identifier)
      let offset = self.offset();
      self.consume(
        &mut children,
        SKIP_ALL_TRIVIA,
        mode,
        SyntaxKind::Ident,
        Diagnostic::MissingSyntaxNode {
          expected: SyntaxKind::Ident,
          start_offset: offset,
          end_offset: self.offset(),
        },
      );
      let key_token = children.pop().unwrap();
      children.push(self.emit(SyntaxKind::DictEntryKey, &[key_token]));
    }

    // Colon
    let peek = self.lex_ctx.peek(SKIP_WS, mode);
    if peek.token.kind() == SyntaxKind::Colon {
      self.advance(&mut children, SKIP_WS, mode);
    } else {
      // Missing colon: emit diagnostic but continue to parse value
      self.diagnostics.push(Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Colon,
        start_offset: self.offset(),
        end_offset: self.offset(),
      });
    }

    // Value
    let peek = self.lex_ctx.peek(SKIP_ALL_TRIVIA, mode);
    match peek.token.kind() {
      SyntaxKind::Comma | SyntaxKind::RBrace | SyntaxKind::Eof => {
        // Missing value
        self.diagnostics.push(Diagnostic::MissingSyntaxNode {
          expected: SyntaxKind::DictEntryValue,
          start_offset: self.offset(),
          end_offset: self.offset(),
        });
        children.push(self.emit(SyntaxKind::DictEntryValue, &[]));
        (self.emit(SyntaxKind::DictEntry, &children), None)
      }
      _ => {
        let (value_expr, early_exit) = self.parse_expr();
        children.push(self.emit(SyntaxKind::DictEntryValue, &[value_expr]));
        (self.emit(SyntaxKind::DictEntry, &children), early_exit)
      }
    }
  }

  // Stop on Comma and RBrace
  fn synchronize_dict_lit(&mut self, children: &mut Vec<GreenNode>) -> Option<ExprCtx> {
    let mut error_children = vec![];
    let result = loop {
      let peek = self.lex_ctx.peek(SKIP_NONE, self.lex_ctx.mode());
      match peek.token.kind() {
        SyntaxKind::Comma | SyntaxKind::RBrace | SyntaxKind::Eof => break None,
        _ => {
          if let Some(ctx) = self.consume_or_delegate(ExprCtx::Dict, &mut error_children) {
            break Some(ctx);
          }
        }
      }
    };
    if !error_children.is_empty() {
      children.push(self.emit(SyntaxKind::Error, &error_children));
    }
    result
  }

  // Stop on `-` (YamlOp), YamlDedent, Newline, Eof
  fn synchronize_block_seq(&mut self, children: &mut Vec<GreenNode>) -> Option<ExprCtx> {
    let mut error_children = vec![];
    let result = loop {
      let peek = self.lex_ctx.peek(SKIP_NONE, self.lex_ctx.mode());
      match peek.token.kind() {
        SyntaxKind::YamlDedent | SyntaxKind::Newline | SyntaxKind::Eof => break None,
        SyntaxKind::YamlOp if peek.token.text().collect::<String>() == "-" => break None,
        _ => {
          if let Some(ctx) = self.consume_or_delegate(ExprCtx::BlockSeq, &mut error_children) {
            break Some(ctx);
          }
        }
      }
    };
    if !error_children.is_empty() {
      children.push(self.emit(SyntaxKind::Error, &error_children));
    }
    result
  }

  // Stop on Ident, Colon, YamlDedent, Newline, Eof
  fn synchronize_block_mapping(&mut self, children: &mut Vec<GreenNode>) -> Option<ExprCtx> {
    let mut error_children = vec![];
    let result = loop {
      let peek = self.lex_ctx.peek(SKIP_NONE, self.lex_ctx.mode());
      match peek.token.kind() {
        SyntaxKind::Ident
        | SyntaxKind::Colon
        | SyntaxKind::YamlDedent
        | SyntaxKind::Newline
        | SyntaxKind::Eof => break None,
        _ => {
          if let Some(ctx) = self.consume_or_delegate(ExprCtx::BlockMap, &mut error_children) {
            break Some(ctx);
          }
        }
      }
    };
    if !error_children.is_empty() {
      children.push(self.emit(SyntaxKind::Error, &error_children));
    }
    result
  }

  /// Parse a literal block string: `|` followed by indented content.
  pub(in crate::parse) fn parse_literal_block_str_lit(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mode = self.lex_ctx.mode();
    debug_assert!(
      {
        let peek = self.lex_ctx.peek(self.expr_skip_flags(), mode);
        peek.token.kind() == SyntaxKind::YamlOp && peek.token.text().collect::<String>() == "|"
      },
      "[ParseCtx::parse_literal_block_str_lit] Expected next token to be `|`"
    );

    let mut children = vec![];

    // Consume `|`
    self.advance(&mut children, self.expr_skip_flags(), mode);

    // Expect newline after `|`
    let offset = self.offset();
    self.consume(
      &mut children,
      SKIP_WS | SKIP_COMMENT,
      mode,
      SyntaxKind::Newline,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Newline,
        start_offset: offset,
        end_offset: self.offset(),
      },
    );

    // Expect indent
    let offset = self.offset();
    self.consume(
      &mut children,
      SKIP_NONE,
      mode,
      SyntaxKind::YamlIndent,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::YamlIndent,
        start_offset: offset,
        end_offset: self.offset(),
      },
    );

    // Consume content until dedent or EOF
    loop {
      let peek = self.lex_ctx.peek(SKIP_NONE, mode);
      match peek.token.kind() {
        SyntaxKind::YamlDedent | SyntaxKind::Eof => break,
        _ => {
          self.advance(&mut children, SKIP_NONE, mode);
        }
      }
    }

    (self.emit(SyntaxKind::LiteralBlockStrLit, &children), None)
  }

  /// Parse a folded block string: `>` followed by indented content.
  pub(in crate::parse) fn parse_folded_block_str_lit(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mode = self.lex_ctx.mode();
    debug_assert!(
      {
        let peek = self.lex_ctx.peek(self.expr_skip_flags(), mode);
        peek.token.kind() == SyntaxKind::YamlOp && peek.token.text().collect::<String>() == ">"
      },
      "[ParseCtx::parse_folded_block_str_lit] Expected next token to be `>`"
    );

    let mut children = vec![];

    // Consume `>`
    self.advance(&mut children, self.expr_skip_flags(), mode);

    // Expect newline after `>`
    let offset = self.offset();
    self.consume(
      &mut children,
      SKIP_WS | SKIP_COMMENT,
      mode,
      SyntaxKind::Newline,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Newline,
        start_offset: offset,
        end_offset: self.offset(),
      },
    );

    // Expect indent
    let offset = self.offset();
    self.consume(
      &mut children,
      SKIP_NONE,
      mode,
      SyntaxKind::YamlIndent,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::YamlIndent,
        start_offset: offset,
        end_offset: self.offset(),
      },
    );

    // Consume content until dedent or EOF
    loop {
      let peek = self.lex_ctx.peek(SKIP_NONE, mode);
      match peek.token.kind() {
        SyntaxKind::YamlDedent | SyntaxKind::Eof => break,
        _ => {
          self.advance(&mut children, SKIP_NONE, mode);
        }
      }
    }

    (self.emit(SyntaxKind::FoldedBlockStrLit, &children), None)
  }

  /// Parse an inline block mapping: starts with `key: value` on the current line.
  /// Continuation entries require an indent on subsequent lines.
  /// INVARIANT: Next token must be Ident followed by Colon.
  fn parse_inline_block_mapping_lit(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mode = self.lex_ctx.mode();
    let mut children = vec![];
    self.expr_ctx_stack.enter(ExprCtx::BlockMap);

    // Parse first entry on the current line
    let (entry, early_exit) = self.parse_block_mapping_entry();
    children.push(entry);
    if early_exit.is_some_and(|ctx| ctx != ExprCtx::BlockMap) {
      self.expr_ctx_stack.exit(ExprCtx::BlockMap);
      return (
        self.emit(SyntaxKind::BlockMappingLit, &children),
        early_exit,
      );
    }

    // Check for continuation entries on indented lines
    let peek = self
      .lex_ctx
      .peek(SKIP_NEWLINE | SKIP_WS | SKIP_COMMENT, mode);
    if peek.token.kind() == SyntaxKind::YamlIndent {
      // Consume indent and parse remaining entries like block_mapping_lit
      self.advance(&mut children, SKIP_NEWLINE | SKIP_WS | SKIP_COMMENT, mode);
      loop {
        let peek = self
          .lex_ctx
          .peek(SKIP_NEWLINE | SKIP_WS | SKIP_COMMENT, mode);

        match peek.token.kind() {
          SyntaxKind::YamlDedent | SyntaxKind::Eof => break,
          SyntaxKind::Ident | SyntaxKind::Colon => {
            let (entry, early_exit) = self.parse_block_mapping_entry();
            children.push(entry);
            if early_exit.is_some_and(|ctx| ctx != ExprCtx::BlockMap) {
              self.expr_ctx_stack.exit(ExprCtx::BlockMap);
              return (
                self.emit(SyntaxKind::BlockMappingLit, &children),
                early_exit,
              );
            }
          }
          _ => {
            let handler = self.expr_ctx_stack.find_handler(&peek.token);
            if handler.is_some_and(|ctx| ctx != ExprCtx::BlockMap) {
              self.expr_ctx_stack.exit(ExprCtx::BlockMap);
              return (self.emit(SyntaxKind::BlockMappingLit, &children), handler);
            }
            if let Some(ctx) = self.synchronize_block_mapping(&mut children) {
              self.expr_ctx_stack.exit(ExprCtx::BlockMap);
              return (self.emit(SyntaxKind::BlockMappingLit, &children), Some(ctx));
            }
          }
        }
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::BlockMap);
    (self.emit(SyntaxKind::BlockMappingLit, &children), None)
  }

  /// Parse a block mapping literal (indentation-based `key: value` pairs).
  pub(in crate::parse) fn parse_block_mapping_lit(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mode = self.lex_ctx.mode();
    let mut children = vec![];
    self.expr_ctx_stack.enter(ExprCtx::BlockMap);

    loop {
      let peek = self
        .lex_ctx
        .peek(SKIP_NEWLINE | SKIP_WS | SKIP_COMMENT, mode);

      match peek.token.kind() {
        SyntaxKind::YamlDedent | SyntaxKind::Eof => break,
        SyntaxKind::Ident | SyntaxKind::Colon => {
          let (entry, early_exit) = self.parse_block_mapping_entry();
          children.push(entry);
          if early_exit.is_some_and(|ctx| ctx != ExprCtx::BlockMap) {
            self.expr_ctx_stack.exit(ExprCtx::BlockMap);
            return (
              self.emit(SyntaxKind::BlockMappingLit, &children),
              early_exit,
            );
          }
        }
        _ => {
          let handler = self.expr_ctx_stack.find_handler(&peek.token);
          if handler.is_some_and(|ctx| ctx != ExprCtx::BlockMap) {
            self.expr_ctx_stack.exit(ExprCtx::BlockMap);
            return (self.emit(SyntaxKind::BlockMappingLit, &children), handler);
          }
          if let Some(ctx) = self.synchronize_block_mapping(&mut children) {
            self.expr_ctx_stack.exit(ExprCtx::BlockMap);
            return (self.emit(SyntaxKind::BlockMappingLit, &children), Some(ctx));
          }
        }
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::BlockMap);
    (self.emit(SyntaxKind::BlockMappingLit, &children), None)
  }

  /// Parse a single block mapping entry: `key: value`.
  fn parse_block_mapping_entry(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mode = self.lex_ctx.mode();
    let mut children = vec![];

    let peek = self
      .lex_ctx
      .peek(SKIP_NEWLINE | SKIP_WS | SKIP_COMMENT, mode);

    // Missing key: `:` seen immediately
    if peek.token.kind() == SyntaxKind::Colon {
      self.diagnostics.push(Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::MappingEntryKey,
        start_offset: self.offset(),
        end_offset: self.offset(),
      });
      children.push(self.emit(SyntaxKind::MappingEntryKey, &[]));
    } else {
      // Key (identifier)
      let offset = self.offset();
      self.consume(
        &mut children,
        SKIP_NEWLINE | SKIP_WS | SKIP_COMMENT,
        mode,
        SyntaxKind::Ident,
        Diagnostic::MissingSyntaxNode {
          expected: SyntaxKind::MappingEntryKey,
          start_offset: offset,
          end_offset: self.offset(),
        },
      );
      let key_token = children.pop().unwrap();
      children.push(self.emit(SyntaxKind::MappingEntryKey, &[key_token]));
    }

    // Colon
    let peek = self.lex_ctx.peek(SKIP_WS, mode);
    if peek.token.kind() == SyntaxKind::Colon {
      self.advance(&mut children, SKIP_WS, mode);
    } else {
      self.diagnostics.push(Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Colon,
        start_offset: self.offset(),
        end_offset: self.offset(),
      });
    }

    // Value: check for newline + indent (nested block) or inline expression
    let peek = self.lex_ctx.peek(SKIP_WS | SKIP_COMMENT, mode);
    match peek.token.kind() {
      // Newline: could be a nested block (seq or mapping)
      SyntaxKind::Newline => {
        // Peek past the newline to see if indent follows
        let peek_after = self
          .lex_ctx
          .peek(SKIP_NEWLINE | SKIP_WS | SKIP_COMMENT, mode);
        if peek_after.token.kind() == SyntaxKind::YamlIndent {
          let (nested, early_exit) = self.parse_block_seq_or_mapping();
          children.push(self.emit(SyntaxKind::MappingEntryValue, &[nested]));
          return (self.emit(SyntaxKind::MappingEntry, &children), early_exit);
        } else {
          // Empty value (newline without indent)
          self.diagnostics.push(Diagnostic::MissingSyntaxNode {
            expected: SyntaxKind::MappingEntryValue,
            start_offset: self.offset(),
            end_offset: self.offset(),
          });
          children.push(self.emit(SyntaxKind::MappingEntryValue, &[]));
        }
      }
      // Dedent or EOF: missing value
      SyntaxKind::YamlDedent | SyntaxKind::Eof => {
        self.diagnostics.push(Diagnostic::MissingSyntaxNode {
          expected: SyntaxKind::MappingEntryValue,
          start_offset: self.offset(),
          end_offset: self.offset(),
        });
        children.push(self.emit(SyntaxKind::MappingEntryValue, &[]));
      }
      // Inline value
      _ => {
        let (value, early_exit) = self.parse_expr();
        children.push(self.emit(SyntaxKind::MappingEntryValue, &[value]));
        return (self.emit(SyntaxKind::MappingEntry, &children), early_exit);
      }
    }

    (self.emit(SyntaxKind::MappingEntry, &children), None)
  }

  /// Parse a double-quoted string literal with interpolation: `"content ${expr} content"`.
  pub(in crate::parse) fn parse_dq_str_lit(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mode = self.lex_ctx.mode();
    debug_assert!(
      self.lex_ctx.peek(self.expr_skip_flags(), mode).token.kind() == SyntaxKind::DqStrStart,
      "[ParseCtx::parse_dq_str_lit] Expected next token to be DqStrStart"
    );

    let mut children = vec![];
    self.expr_ctx_stack.enter(ExprCtx::DqString);
    self.advance(&mut children, self.expr_skip_flags(), mode);

    loop {
      let peek = self.lex_ctx.peek(SKIP_NONE, mode);
      match peek.token.kind() {
        SyntaxKind::DqStrEnd => {
          self.advance(&mut children, SKIP_NONE, mode);
          break;
        }
        SyntaxKind::InterpStart => {
          let (fragment, early_exit) = self.parse_interp_fragment();
          children.push(fragment);
          if early_exit.is_some_and(|ctx| ctx != ExprCtx::DqString) {
            self.expr_ctx_stack.exit(ExprCtx::DqString);
            return (self.emit(SyntaxKind::StrLit, &children), early_exit);
          }
        }
        SyntaxKind::InlineMath => {
          let (math, early_exit) = self.parse_math_lit();
          children.push(math);
          if early_exit.is_some_and(|ctx| ctx != ExprCtx::DqString) {
            self.expr_ctx_stack.exit(ExprCtx::DqString);
            return (self.emit(SyntaxKind::StrLit, &children), early_exit);
          }
        }
        SyntaxKind::Eof | SyntaxKind::Error => {
          self.advance(&mut children, SKIP_NONE, mode);
          break;
        }
        _ => {
          self.advance(&mut children, SKIP_NONE, mode);
        }
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::DqString);
    (self.emit(SyntaxKind::StrLit, &children), None)
  }

  /// Parse a single-quoted string literal with interpolation: `'content ${expr} content'`.
  pub(in crate::parse) fn parse_sq_str_lit(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mode = self.lex_ctx.mode();
    debug_assert!(
      self.lex_ctx.peek(self.expr_skip_flags(), mode).token.kind() == SyntaxKind::SqStrStart,
      "[ParseCtx::parse_sq_str_lit] Expected next token to be SqStrStart"
    );

    let mut children = vec![];
    self.expr_ctx_stack.enter(ExprCtx::SqString);
    self.advance(&mut children, self.expr_skip_flags(), mode);

    loop {
      let peek = self.lex_ctx.peek(SKIP_NONE, mode);
      match peek.token.kind() {
        SyntaxKind::SqStrEnd => {
          self.advance(&mut children, SKIP_NONE, mode);
          break;
        }
        SyntaxKind::InterpStart => {
          let (fragment, early_exit) = self.parse_interp_fragment();
          children.push(fragment);
          if early_exit.is_some_and(|ctx| ctx != ExprCtx::SqString) {
            self.expr_ctx_stack.exit(ExprCtx::SqString);
            return (self.emit(SyntaxKind::StrLit, &children), early_exit);
          }
        }
        SyntaxKind::InlineMath => {
          let (math, early_exit) = self.parse_math_lit();
          children.push(math);
          if early_exit.is_some_and(|ctx| ctx != ExprCtx::SqString) {
            self.expr_ctx_stack.exit(ExprCtx::SqString);
            return (self.emit(SyntaxKind::StrLit, &children), early_exit);
          }
        }
        SyntaxKind::Eof | SyntaxKind::Error => {
          self.advance(&mut children, SKIP_NONE, mode);
          break;
        }
        _ => {
          self.advance(&mut children, SKIP_NONE, mode);
        }
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::SqString);
    (self.emit(SyntaxKind::StrLit, &children), None)
  }

  /// Parse an interpolation fragment: `${...}` inside a string.
  pub(in crate::parse) fn parse_interp_fragment(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mode = self.lex_ctx.mode();
    debug_assert!(
      self.lex_ctx.peek(SKIP_NONE, mode).token.kind() == SyntaxKind::InterpStart,
      "[ParseCtx::parse_interp_fragment] Expected next token to be InterpStart"
    );

    let mut children = vec![];
    self.expr_ctx_stack.enter(ExprCtx::Interp);

    // Consume `${`
    self.advance(&mut children, SKIP_NONE, mode);

    // Parse the expression inside
    let (inner, early_exit) = self.parse_formula_expr();
    children.push(inner);

    if early_exit.is_some_and(|ctx| ctx != ExprCtx::Interp) {
      self.expr_ctx_stack.exit(ExprCtx::Interp);
      return (self.emit(SyntaxKind::InterpFragment, &children), early_exit);
    }

    // Consume `}`
    let offset = self.offset();
    self.consume(
      &mut children,
      SKIP_ALL_TRIVIA,
      mode,
      SyntaxKind::InterpEnd,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::InterpEnd,
        start_offset: offset,
        end_offset: self.offset(),
      },
    );

    self.expr_ctx_stack.exit(ExprCtx::Interp);
    (self.emit(SyntaxKind::InterpFragment, &children), None)
  }

  /// Parse a math literal (inline or block math).
  /// Wraps a single InlineMath or MathBlock token.
  pub(in crate::parse) fn parse_math_lit(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mode = self.lex_ctx.mode();
    debug_assert!(
      matches!(
        self.lex_ctx.peek(self.expr_skip_flags(), mode).token.kind(),
        SyntaxKind::InlineMath | SyntaxKind::MathBlock
      ),
      "[ParseCtx::parse_math_lit] Expected next token to be InlineMath or MathBlock"
    );
    let mut children = vec![];
    self.advance(&mut children, self.expr_skip_flags(), mode);
    (self.emit(SyntaxKind::MathLit, &children), None)
  }

  /// Parse a code literal (inline or block code).
  /// Wraps a single InlineCode or CodeBlock token.
  pub(in crate::parse) fn parse_code_lit(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mode = self.lex_ctx.mode();
    debug_assert!(
      matches!(
        self.lex_ctx.peek(self.expr_skip_flags(), mode).token.kind(),
        SyntaxKind::InlineCode | SyntaxKind::CodeBlock
      ),
      "[ParseCtx::parse_code_lit] Expected next token to be InlineCode or CodeBlock"
    );
    let mut children = vec![];
    self.advance(&mut children, self.expr_skip_flags(), mode);
    (self.emit(SyntaxKind::CodeLit, &children), None)
  }

  /// Parse a number literal.
  /// Wraps a single Number token.
  pub(in crate::parse) fn parse_number_lit(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mode = self.lex_ctx.mode();
    debug_assert!(
      self.lex_ctx.peek(self.expr_skip_flags(), mode).token.kind() == SyntaxKind::Number,
      "[ParseCtx::parse_number_lit] Expected next token to be Number"
    );
    let mut children = vec![];
    self.advance(&mut children, self.expr_skip_flags(), mode);
    (self.emit(SyntaxKind::NumberLit, &children), None)
  }

  /// Parse an identifier literal.
  /// Wraps a single Ident token.
  pub(in crate::parse) fn parse_ident_lit(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mode = self.lex_ctx.mode();
    debug_assert!(
      self.lex_ctx.peek(self.expr_skip_flags(), mode).token.kind() == SyntaxKind::Ident,
      "[ParseCtx::parse_ident_lit] Expected next token to be Ident"
    );
    let mut children = vec![];
    self.advance(&mut children, self.expr_skip_flags(), mode);
    (self.emit(SyntaxKind::IdentLit, &children), None)
  }

  /// If the next token should be handled by an outer context, return that context.
  /// Otherwise consume the token into `error_children` for the caller to wrap.
  fn consume_or_delegate(
    &mut self,
    current: ExprCtx,
    error_children: &mut Vec<GreenNode>,
  ) -> Option<ExprCtx> {
    let peek = self.lex_ctx.peek(SKIP_NONE, self.lex_ctx.mode());
    let handler = self.expr_ctx_stack.find_handler(&peek.token);
    if handler.is_some_and(|ctx| ctx != current) {
      return handler;
    }
    let mode = self.lex_ctx.mode();
    self.advance(error_children, SKIP_NONE, mode);
    None
  }
}

pub(in crate::parse) fn prefix_binding_power(op: &str) -> Option<((), u8)> {
  let bp = match op {
    _ if op.starts_with('!') => 1,
    "~" | "-" | "+" => 15,
    _ => return None,
  };
  Some(((), bp))
}

pub(in crate::parse) fn infix_binding_power(op: &str) -> Option<(u8, u8)> {
  let bp = match op {
    "||" => (3, 4),                     // logical OR
    "&&" => (5, 6),                     // logical AND
    "==" | "!=" => (7, 8),              // equality
    "<" | ">" | "<=" | ">=" => (9, 10), // comparison
    "+" | "-" => (11, 12),              // additive
    "*" | "/" | "%" => (13, 14),        // multiplicative
    "**" => (18, 17),                   // exponentiation (right-assoc)
    "." => (19, 20),                    // member access
    _ => return None,
  };
  Some(bp)
}

pub(in crate::parse) fn postfix_binding_power(op: &str) -> Option<(u8, ())> {
  None
}
