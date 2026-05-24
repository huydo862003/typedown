//! Parser method for parsing many kinds of expressions

use typedown_types::{diagnostic::Diagnostic, stream::Utf8Stream, syntax_kind::SyntaxKind};

use super::constants::*;
use crate::{
  green::GreenNode,
  parse::ctx::{ParseCtx, expr_ctx::ExprCtx},
};

impl<S: Utf8Stream> ParseCtx<S> {
  /// General expression, including formula and yaml.
  pub(in crate::parse) fn parse_expr(&mut self) -> (GreenNode, Option<ExprCtx>) {
    todo!()
  }

  /// Formula expressions: Pratt-parsed expressions that follow most programming language rules
  pub(in crate::parse) fn parse_formula_expr(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a primary expression (an operand): literal, ident, paren, etc.
  pub(in crate::parse) fn parse_primary_expr(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a parenthesized expression: `(expr)`.
  pub(in crate::parse) fn parse_paren_expr(&mut self) -> GreenNode {
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
    let inner = self.parse_formula_expr();
    children.push(inner);

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

    self.emit(SyntaxKind::ParenExpr, &children)
  }

  /// Parse a tagged literal: `!tag value`.
  pub(in crate::parse) fn parse_tagged_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a flow list literal: `[expr, expr, ...]`.
  pub(in crate::parse) fn parse_list_lit(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let outer_skip = SKIP_WCN
      | if self.expr_ctx_stack.should_ignore_indent() {
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
          let handler = self.expr_ctx_stack.find_handler(peek.token.kind());
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

  /// Parse a block sequence literal: lines starting with `-`.
  pub(in crate::parse) fn parse_block_seq_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a flow mapping literal: `{key: value, ...}`.
  pub(in crate::parse) fn parse_mapping_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a literal block string: `|` followed by indented content.
  pub(in crate::parse) fn parse_literal_block_str_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a folded block string: `>` followed by indented content.
  pub(in crate::parse) fn parse_folded_block_str_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a block mapping literal (delegates to yaml block mapping).
  pub(in crate::parse) fn parse_block_mapping_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a string literal (double or single quoted, with interpolation).
  pub(in crate::parse) fn parse_str_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse an interpolation fragment: `${...}` inside a string.
  pub(in crate::parse) fn parse_interp_fragment(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a math literal (inline or block math).
  pub(in crate::parse) fn parse_math_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a code literal (inline or block code).
  pub(in crate::parse) fn parse_code_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a number literal.
  pub(in crate::parse) fn parse_number_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse an identifier literal.
  pub(in crate::parse) fn parse_ident_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a tag: `!name`.
  pub(in crate::parse) fn parse_tag(&mut self) -> GreenNode {
    todo!()
  }

  /// If the next token should be handled by an outer context, return that context.
  /// Otherwise consume the token into `error_children` for the caller to wrap.
  fn consume_or_delegate(
    &mut self,
    current: ExprCtx,
    error_children: &mut Vec<GreenNode>,
  ) -> Option<ExprCtx> {
    let peek = self.lex_ctx.peek(SKIP_NONE, self.lex_ctx.mode());
    let handler = self.expr_ctx_stack.find_handler(peek.token.kind());
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
    "!" | "-" | "+" => 15,
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
