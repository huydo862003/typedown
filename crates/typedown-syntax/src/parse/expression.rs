//! Parser method for parsing many kinds of expressions

use typedown_types::{diagnostic::Diagnostic, stream::Utf8Stream, syntax_kind::SyntaxKind};

use super::constants::*;
use crate::{green::GreenNode, lex::ctx::LexMode, parse::ctx::ParseCtx};

impl<S: Utf8Stream> ParseCtx<S> {
  /// General expression, including formula and yaml
  pub(super) fn parse_expression(&mut self) -> GreenNode {
    todo!()
  }

  /// Formula expressions: Pratt-parsed expressions that follow most programming language rules
  pub(super) fn parse_formula_expression(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a primary expression (an operand): literal, ident, paren, etc.
  pub(super) fn parse_primary_expr(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a parenthesized expression: `(expr)`.
  pub(super) fn parse_paren_expr(&mut self) -> GreenNode {
    debug_assert!(
      self
        .lex_ctx
        .peek(SKIP_WCN, self.lex_ctx.mode())
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
      SKIP_WCN,
      self.lex_ctx.mode(),
      SyntaxKind::LParen,
      Diagnostic::MissingToken {
        expected: SyntaxKind::LParen,
        start_offset: offset,
        end_offset: self.offset(),
      },
    );

    // Parse inner expression
    let inner = self.parse_formula_expression();
    children.push(inner);

    // Consume `)`
    let offset = self.offset();
    self.consume(
      &mut children,
      SKIP_WCN,
      self.lex_ctx.mode(),
      SyntaxKind::RParen,
      Diagnostic::MissingToken {
        expected: SyntaxKind::RParen,
        start_offset: offset,
        end_offset: self.offset(),
      },
    );

    self.emit(SyntaxKind::ParenExpr, &children)
  }

  /// Parse a unary expression: `!expr`, `-expr`, `+expr`.
  pub(super) fn parse_unary_expr(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a binary expression (handled by Pratt parser).
  pub(super) fn parse_binary_expr(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a tagged literal: `!tag value`.
  pub(super) fn parse_tagged_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a flow list literal: `[expr, expr, ...]`.
  pub(super) fn parse_list_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a block sequence literal: lines starting with `-`.
  pub(super) fn parse_block_seq_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a flow mapping literal: `{key: value, ...}`.
  pub(super) fn parse_mapping_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a literal block string: `|` followed by indented content.
  pub(super) fn parse_literal_block_str_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a folded block string: `>` followed by indented content.
  pub(super) fn parse_folded_block_str_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a block mapping literal (delegates to yaml block mapping).
  pub(super) fn parse_block_mapping_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a string literal (double or single quoted, with interpolation).
  pub(super) fn parse_str_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse an interpolation fragment: `${...}` inside a string.
  pub(super) fn parse_interp_fragment(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a math literal (inline or block math).
  pub(super) fn parse_math_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a code literal (inline or block code).
  pub(super) fn parse_code_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a number literal.
  pub(super) fn parse_number_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse an identifier literal.
  pub(super) fn parse_ident_lit(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a tag: `!name`.
  pub(super) fn parse_tag(&mut self) -> GreenNode {
    todo!()
  }
}

pub(super) fn prefix_binding_power(op: &str) -> Option<((), u8)> {
  let bp = match op {
    "!" | "-" | "+" => 15,
    _ => return None,
  };
  Some(((), bp))
}

pub(super) fn infix_binding_power(op: &str) -> Option<(u8, u8)> {
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

pub(super) fn postfix_binding_power(op: &str) -> Option<(u8, ())> {
  None
}
