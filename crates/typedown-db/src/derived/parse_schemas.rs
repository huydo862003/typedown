//! Tracked query to parse all schema files in the vault's schema directory

use std::collections::HashMap;

use typedown_macros::query_derived;

use crate::{
  QueryDatabase, TypedownDatabase,
  types::{File, Project, SchemaAstResults},
};

use super::get_vault_config::get_vault_config;
use super::parse_file::parse_file;

#[query_derived]
pub fn parse_schemas(db: &TypedownDatabase, project: Project) -> SchemaAstResults {
  let config = get_vault_config(db, project);
  let schema_dir = config.schema_dir(db);
  let handles = project.handles(db);

  let mut files = HashMap::new();

  for (path, handle) in &handles {
    if path.starts_with(&schema_dir) && path.extension().is_some_and(|ext| ext == "tdr") {
      let file = File::new(db, handle.clone());
      let ast = parse_file(db, project, file);
      files.insert(path.clone(), ast);
    }
  }

  SchemaAstResults::new(db, files)
}
