//! Derived types for the incremental database

use std::collections::HashMap;
use std::path::PathBuf;

use typedown_macros::query_derived;
use typedown_types::diagnostic::Diagnostic;

use crate::types::GreenNode;

use super::inputs::FileHandle;

#[query_derived]
pub struct VaultConfig {
  version: String,
  content_dir: PathBuf,
  schema_dir: PathBuf,
}

#[query_derived]
pub struct FileAst {
  #[id]
  handle: FileHandle,
  ast: GreenNode,
  diagnostics: Vec<Diagnostic>,
}

#[query_derived]
pub struct SchemaAsts {
  files: HashMap<PathBuf, FileAst>,
}
