//! Derived types for the incremental database

pub mod object_system;

pub use object_system::*;

use std::collections::HashMap;
use std::path::PathBuf;

use typedown_macros::query_derived;
use typedown_types::diagnostic::Diagnostic;

use typedown_syntax::red::RedNode;

use super::inputs::{File, FileHandle, Project};

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
  project: Project,
  file: File,
  ast: RedNode,
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
  typ: Box<dyn TdrTypeLike>,
  diagnostics: Vec<Diagnostic>,
}

#[query_derived]
pub struct InstResult {
  pub typ: Box<dyn TdrTypeLike>,
  pub diagnostics: Vec<Diagnostic>,
}
