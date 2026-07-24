//! Derived types for the incremental database

pub mod hir;
pub mod object_system;
pub mod symbol;

pub use hir::*;
pub use object_system::*;
pub use symbol::*;

use std::collections::HashMap;
use std::path::PathBuf;

use crate::{db::types::TypeMember, syntax::diagnostic::Diagnostic};
use tdr_incremental::{
  Decodable, Decoder, Encodable, Encoder, QueryDatabase, StableHash, StableHasher,
};
use tdr_macros::query_derived;

use crate::syntax::red::RedNode;

use super::inputs::{File, FileHandle, Project};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AssetsDirMode {
  Local,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetsDir {
  pub mode: AssetsDirMode,
  pub path: String,
}

impl Default for AssetsDir {
  fn default() -> Self {
    AssetsDir {
      mode: AssetsDirMode::Local,
      path: "assets".to_string(),
    }
  }
}

impl StableHash for AssetsDir {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    std::mem::discriminant(&self.mode).stable_hash(db, hasher);
    self.path.stable_hash(db, hasher);
  }
}

impl Encodable for AssetsDir {
  fn encode(&self, buf: &mut Vec<u8>, encoder: &mut Encoder) {
    encoder.emit_u8(buf, 0); // Local = 0
    self.path.encode(buf, encoder);
  }
}

impl Decodable for AssetsDir {
  fn decode(data: &mut &[u8], decoder: &Decoder) -> Self {
    let _mode = decoder.read_u8(data); // Only Local for now
    let path = String::decode(data, decoder);
    AssetsDir {
      mode: AssetsDirMode::Local,
      path,
    }
  }
}

#[query_derived]
pub struct VaultConfigResult {
  version: String,
  content_dir: PathBuf,
  schema_dir: PathBuf,
  base_path: String,
  assets_dir: AssetsDir,
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
