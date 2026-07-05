use typedown_macros::query_derived;

use crate::db::TypedownDatabase;
use crate::db::derived::get_vault_config::get_vault_config;
use crate::db::types::{File, FileHandle, Project, Symbol, SymbolKind};
use typedown_incremental::QueryDatabase;

#[query_derived]
pub struct MaybeSymbol {
  pub value: Option<Symbol>,
}

#[query_derived]
pub fn file_symbol(db: &TypedownDatabase, project: Project, file: File) -> MaybeSymbol {
  let config = get_vault_config(db, project);
  let schema_dir = config.schema_dir(db);
  let proj_files = project.files(db);

  let is_schema_file = proj_files
    .iter()
    .any(|(path, proj_file)| *proj_file == file && path.starts_with(&schema_dir));

  let name = match file.handle(db) {
    FileHandle::Path(path, _) => path
      .file_stem()
      .and_then(|s| s.to_str())
      .unwrap_or_default()
      .to_string(),
    FileHandle::Content(_) => String::new(),
  };

  let kind = if is_schema_file {
    SymbolKind::UserDefinedSchema(project, file)
  } else {
    SymbolKind::UserDefinedResource(project, file)
  };

  MaybeSymbol::new(db, Some(Symbol::new(db, kind, name)))
}
