//! Derived types for the incremental database

use std::collections::HashMap;
use std::path::PathBuf;

use typedown_macros::query_derived;
use typedown_types::diagnostic::Diagnostic;

use crate::types::GreenNode;

use super::inputs::FileHandle;

#[query_derived]
pub struct VaultConfigResult {
  version: String,
  content_dir: PathBuf,
  schema_dir: PathBuf,
  diagnostics: Vec<Diagnostic>,
}

#[query_derived]
pub struct FileAstResult {
  #[id]
  handle: FileHandle,
  ast: GreenNode,
  diagnostics: Vec<Diagnostic>,
}

#[query_derived]
pub struct SchemaAstResults {
  files: HashMap<PathBuf, FileAstResult>,
}

#[query_derived]
pub struct TypecheckResult {
  diagnostics: Vec<Diagnostic>,
}

#[query_derived]
pub struct TypeResult {
  diagnostics: Vec<Diagnostic>,
}
