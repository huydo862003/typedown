use tdr_lang::{
  db::{
    TypedownDatabase,
    derived::parse_file::parse_file,
    types::{File, Project},
  },
  syntax::{
    ast::{AstNode, IdentLit},
    red::RedNode,
    syntax_kind::SyntaxKind,
  },
};

use crate::{
  service::rename::types::RenameSymbol,
  utils::ast::{containing_fref_expr, find_ancestor, node_at_offset},
};

pub fn find_rename_symbol(
  db: &TypedownDatabase,
  project: Project,
  file: File,
  offset: usize,
) -> Option<RenameSymbol> {
  let root = parse_file(db, project, file).ast(db);
  let node = node_at_offset(root, offset)?;

  // The user can try to rename an fref string
  if let Some(symbol) = find_rename_symbol_as_fref(&node) {
    return Some(symbol);
  };

  // The user try to rename an identifier
  if let Some(symbol) = find_rename_symbol_as_identifier(&node) {
    return Some(symbol);
  }

  None
}

fn find_rename_symbol_as_fref(node: &RedNode) -> Option<RenameSymbol> {
  let call_expr = containing_fref_expr(node)?;
  call_expr.arg(0).and_then(|a| {
    Some(RenameSymbol::Fref {
      string_node: a.try_into().ok()?,
    })
  })
}

fn find_rename_symbol_as_identifier(node: &RedNode) -> Option<RenameSymbol> {
  find_ancestor(node, SyntaxKind::IdentLit).and_then(|i| {
    Some(RenameSymbol::Identifier {
      ident_node: IdentLit::cast(i)?,
    })
  })
}
