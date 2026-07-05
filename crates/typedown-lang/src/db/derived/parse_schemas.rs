//! Tracked query to parse all schema files in the vault's schema directory

use std::collections::HashMap;

use typedown_macros::query_derived;

use crate::db::TypedownDatabase;
use crate::db::types::{Project, SchemaAstResults};
use typedown_incremental::QueryDatabase;

use super::get_vault_config::get_vault_config;
use super::parse_file::parse_file;

#[query_derived]
pub fn parse_schemas(db: &TypedownDatabase, project: Project) -> SchemaAstResults {
  let config = get_vault_config(db, project);
  let schema_dir = config.schema_dir(db);
  let proj_files = project.files(db);

  let mut schema_asts = HashMap::new();

  for (path, file) in &proj_files {
    if path.starts_with(&schema_dir) && path.extension().is_some_and(|ext| ext == "tdr") {
      let ast = parse_file(db, project, *file);
      schema_asts.insert(path.clone(), ast);
    }
  }

  SchemaAstResults::new(db, schema_asts)
}
