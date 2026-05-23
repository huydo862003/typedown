//! Parser method for parsing many kinds of expressions

use typedown_types::stream::Utf8Stream;

use crate::{green::GreenNode, parse::ctx::ParseCtx};

impl<S: Utf8Stream> ParseCtx<S> {
  /// Formula expressions: Pratt-parsed expressions inside `${...}`.
  pub(super) fn parse_formula_expression(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a `!tag` token (e.g. `!string`, `!number`).
  pub(super) fn parse_tag_expression(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a tagged expression: `!tag value` (e.g. `!string "hello"`).
  pub(super) fn parse_tagged_expression(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a primary expression: literal, identifier, string, parenthesized, etc.
  pub(super) fn parse_primary_expression(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a string expression (double or single quoted, with interpolation).
  pub(super) fn parse_string_expression(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a parenthesized expression: `(expr)`.
  pub(super) fn parse_paren_expression(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a folded block string: `>` followed by indented content.
  pub(super) fn parse_folded_string(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a literal block string: `|` followed by indented content.
  pub(super) fn parse_literal_string(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a block sequence: lines starting with `-`.
  pub(super) fn parse_block_seq(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a single block sequence item: `- value`.
  pub(super) fn parse_block_seq_item(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a flow sequence: `[expr, expr, ...]`.
  pub(super) fn parse_flow_seq(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a block mapping (delegates to yaml.rs implementation).
  pub(super) fn parse_block_mapping(&mut self) -> GreenNode {
    todo!()
  }

  /// Parse a flow mapping: `{key: value, ...}`.
  pub(super) fn parse_flow_mapping(&mut self) -> GreenNode {
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
