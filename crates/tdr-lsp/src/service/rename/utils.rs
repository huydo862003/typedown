use tdr_lang::db::{
  TypedownDatabase,
  types::{File, Project},
};

use crate::service::rename::types::RenameSymbol;

pub fn find_rename_symbol(
  db: &TypedownDatabase,
  project: Project,
  file: File,
  offset: usize,
) -> Option<RenameSymbol> {
  todo!()
}
