use crate::utils::position::text_offset_to_lsp_position;
use lsp_types::Range;
use ropey::Rope;
use tdr_lang::syntax::ast::{AstNode, CallExpr, IdentLit};

use super::utils::str_content_node;

pub enum RenameSymbol {
  Fref { call_node: CallExpr },
  Identifier { ident_node: IdentLit },
}

impl RenameSymbol {
  pub fn get_range(&self, rope: &Rope) -> Range {
    let (offset, len) = match self {
      RenameSymbol::Fref { call_node } => {
        // Return the range of the string content (minus quotes)
        let arg = call_node.arg(0).expect("fref must have an argument");
        let content =
          str_content_node(arg.syntax()).expect("fref argument must be a string literal");
        content.trimmed_range()
      }
      RenameSymbol::Identifier { ident_node } => ident_node.syntax().trimmed_range(),
    };
    Range {
      start: text_offset_to_lsp_position(rope, offset),
      end: text_offset_to_lsp_position(rope, offset + len),
    }
  }
}
