use lsp_types::Range;
use ropey::Rope;
use tdr_lang::syntax::ast::{AstNode, IdentLit, StrLit};

use crate::utils::position::text_offset_to_lsp_position;

pub enum RenameSymbol {
  Fref { string_node: StrLit },
  Identifier { ident_node: IdentLit },
}

impl RenameSymbol {
  pub fn get_range(&self, rope: &Rope) -> Range {
    match self {
      RenameSymbol::Fref { string_node } => Range {
        start: text_offset_to_lsp_position(rope, string_node.syntax().offset()),
        end: text_offset_to_lsp_position(
          rope,
          string_node.syntax().offset() + string_node.syntax().text_len(),
        ),
      },
      RenameSymbol::Identifier { ident_node } => Range {
        start: text_offset_to_lsp_position(rope, ident_node.syntax().offset()),
        end: text_offset_to_lsp_position(
          rope,
          ident_node.syntax().offset() + ident_node.syntax().text_len(),
        ),
      },
    }
  }
}
