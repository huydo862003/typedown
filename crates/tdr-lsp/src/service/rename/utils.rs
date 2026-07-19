use tdr_lang::{
  db::{
    TypedownDatabase,
    derived::parse_file::parse_file,
    types::{File, Project},
  },
  syntax::red::RedNode,
};

use crate::{service::rename::types::RenameSymbol, utils::ast::node_at_offset};

pub fn find_rename_symbol(
  db: &TypedownDatabase,
  project: Project,
  file: File,
  offset: usize,
) -> Option<RenameSymbol> {
  let root = parse_file(db, project, file).ast(db);
  let node = node_at_offset(root, offset)?;

  // The user can try to rename an fref string
  if let Some(symbol) = find_rename_symbol_as_fref(db, project, file, &node) {
    return Some(symbol);
  };

  // The user try to rename an identifier
  if let Some(symbol) = find_rename_symbol_as_identifier(db, project, file, &node) {
    return Some(symbol);
  }

  None
}

fn find_rename_symbol_as_fref(
  db: &TypedownDatabase,
  project: Project,
  file: File,
  node: &RedNode,
) -> Option<RenameSymbol> {
  todo!()
}

fn find_rename_symbol_as_identifier(
  db: &TypedownDatabase,
  project: Project,
  file: File,
  node: &RedNode,
) -> Option<RenameSymbol> {
  todo!();
}
