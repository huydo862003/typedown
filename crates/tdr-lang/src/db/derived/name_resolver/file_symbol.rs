use tdr_macros::query_derived;
use tdr_types::path::normalize_path;

use crate::db::TypedownDatabase;
use crate::db::derived::get_vault_config::get_vault_config;
use crate::db::types::{File, Project, Symbol, SymbolKind};
use tdr_incremental::QueryDatabase;

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

  let path = file.handle(db).path().cloned().unwrap_or_default();

  let name = path
    .file_stem()
    .and_then(|s| s.to_str())
    .unwrap_or_default()
    .to_string();

  let kind = if is_schema_file {
    SymbolKind::UserDefinedSchema(project, file)
  } else {
    SymbolKind::UserDefinedResource(project, file)
  };

  let root = project.root_dir(db);
  let relative = path.strip_prefix(&root).unwrap_or(&path);
  let def_id = format!("@vault::{}", normalize_path(relative));

  MaybeSymbol::new(db, Some(Symbol::new(db, kind, name, def_id)))
}
