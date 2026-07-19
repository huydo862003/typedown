//! Derived types for the incremental database

pub mod hir;
pub mod object_system;

pub use hir::*;
pub use object_system::*;

use std::collections::HashMap;
use std::path::PathBuf;

use crate::{db::types::TypeMember, syntax::diagnostic::Diagnostic};
use tdr_macros::query_derived;

use crate::syntax::red::RedNode;

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
pub struct ResolveResult {
  diagnostics: Vec<Diagnostic>,
}

#[query_derived]
pub struct TypeResult {
  typ: Option<TdrTypeEnum>,
  diagnostics: Vec<Diagnostic>,
}

#[query_derived]
pub struct TypeMemberResult {
  member: Option<TypeMember>,
  diagnostics: Vec<Diagnostic>,
}

#[query_derived]
pub struct InstResult {
  pub typ: TdrTypeEnum,
  pub diagnostics: Vec<Diagnostic>,
}

#[query_derived]
pub struct ResourceResult {
  pub value: Option<TdrObjectEnum>,
  pub diagnostics: Vec<Diagnostic>,
}
