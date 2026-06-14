//! Derived types for the incremental database

pub mod object_system;

pub use object_system::*;

use std::collections::HashMap;
use std::path::PathBuf;

use typedown_macros::query_derived;
use typedown_types::diagnostic::Diagnostic;

use typedown_syntax::{ast::AstNode, red::RedNode};

use crate::QueryDatabase;
use super::inputs::{File, FileHandle, Project};

#[query_derived]
pub struct TdrNode {
  #[id]
  project: Project,
  #[id]
  file: File,
  node: RedNode,
}

impl TdrNode {
  pub fn try_cast<T: AstNode>(&self, db: &(impl QueryDatabase + ?Sized)) -> Option<T> {
    T::cast(self.node(db))
  }
}

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
  ast: TdrNode,
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
