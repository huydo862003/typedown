use lsp_types::Range;
use ropey::Rope;
use tdr_lang::syntax::ast::{AstNode, CallExpr, IdentLit};

use crate::utils::position::text_offset_to_lsp_position;

pub enum RenameSymbol {
  Fref { call_node: CallExpr },
  Identifier { ident_node: IdentLit },
}

impl RenameSymbol {
  pub fn get_range(&self, rope: &Rope) -> Range {
    let (offset, len) = match self {
      RenameSymbol::Fref { call_node } => call_node.syntax().trimmed_range(),
      RenameSymbol::Identifier { ident_node } => ident_node.syntax().trimmed_range(),
    };
    Range {
      start: text_offset_to_lsp_position(rope, offset),
      end: text_offset_to_lsp_position(rope, offset + len),
    }
  }
}
