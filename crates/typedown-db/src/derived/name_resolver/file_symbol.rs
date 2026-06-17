use typedown_macros::query_derived;

use crate::derived::get_vault_config::get_vault_config;
use crate::types::{File, FileHandle, Project, Symbol, SymbolKind};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub struct MaybeSymbol {
  pub value: Option<Symbol>,
}

#[query_derived]
pub fn file_symbol(db: &TypedownDatabase, project: Project, file: File) -> MaybeSymbol {
  let config = get_vault_config(db, project);
  let schema_dir = config.schema_dir(db);
  let handles = project.handles(db);
  let file_handle = file.handle(db);

  let is_schema_file = handles
    .iter()
    .any(|(path, handle)| *handle == file_handle && path.starts_with(&schema_dir));

  if is_schema_file {
    let name = match file.handle(db) {
      FileHandle::Path(path) => path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string(),
      FileHandle::Content(_) => String::new(),
    };
    return MaybeSymbol::new(
      db,
      Some(Symbol::new(
        db,
        SymbolKind::UserDefinedSchema(project, file),
        name,
      )),
    );
  }

  MaybeSymbol::new(db, None)
}
