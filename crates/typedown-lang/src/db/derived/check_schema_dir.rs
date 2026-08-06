//! Check that schema files are direct children of the schema directory

use typedown_macros::query_derived;
use typedown_types::path::normalize_path;

use crate::db::TypedownDatabase;
use crate::db::derived::get_vault_config::get_vault_config;
use crate::db::types::Project;
use crate::db::utils::is_content_file;
use crate::syntax::diagnostic::Diagnostic;
use typedown_incremental::QueryDatabase;

#[query_derived]
pub struct SchemaCheckResult {
  diagnostics: Vec<Diagnostic>,
}

/// Check all files in schema_dir for unsupported nesting
#[query_derived]
pub fn check_schema_dir(db: &TypedownDatabase, project: Project) -> SchemaCheckResult {
  let config = get_vault_config(db, project);
  let schema_dir = config.schema_dir(db);
  let proj_files = project.files(db);
  let mut diagnostics = vec![];

  for path in proj_files.keys() {
    if !path.starts_with(&schema_dir) {
      continue;
    }
    if !is_content_file(path) {
      continue;
    }
    // Check if the file is a direct child of schema_dir
    if path.parent() != Some(&schema_dir) {
      let relative = path.strip_prefix(&schema_dir).unwrap_or(path);
      diagnostics.push(Diagnostic::NestedSchemaFile {
        path: normalize_path(relative),
      });
    }
  }

  SchemaCheckResult::new(db, diagnostics)
}
