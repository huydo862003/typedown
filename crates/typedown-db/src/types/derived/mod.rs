//! Derived types for the incremental database

pub mod hir;
pub mod object_system;

pub use hir::*;
pub use object_system::*;

use std::collections::HashMap;
use std::path::PathBuf;

use typedown_macros::query_derived;
use typedown_types::diagnostic::Diagnostic;

use typedown_syntax::red::RedNode;

use super::inputs::{File, FileHandle, Project};
use crate::{Decodable, Decoder, Encodable, Encoder, StableHash, StableHasher, TypedownDatabase};

#[query_derived]
pub struct VaultConfigResult {
  version: String,
  content_dir: PathBuf,
  schema_dir: PathBuf,
  diagnostics: Vec<Diagnostic>,
}

impl StableHash<TypedownDatabase> for VaultConfigResult {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.version(db).stable_hash(db, hasher);
    self.content_dir(db).stable_hash(db, hasher);
    self.schema_dir(db).stable_hash(db, hasher);
    self.diagnostics(db).stable_hash(db, hasher);
  }
}

impl Encodable<TypedownDatabase> for VaultConfigResult {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    self.version(encoder.db).encode(encoder);
    self.content_dir(encoder.db).encode(encoder);
    self.schema_dir(encoder.db).encode(encoder);
    self.diagnostics(encoder.db).encode(encoder);
  }
}

impl Decodable<TypedownDatabase> for VaultConfigResult {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    let version = String::decode(decoder);
    let content_dir = PathBuf::decode(decoder);
    let schema_dir = PathBuf::decode(decoder);
    let diagnostics = Vec::decode(decoder);
    VaultConfigResult::new(decoder.db, version, content_dir, schema_dir, diagnostics)
  }
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

impl StableHash<TypedownDatabase> for FileAstResult {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.handle(db).stable_hash(db, hasher);
    self.project(db).stable_hash(db, hasher);
    self.file(db).stable_hash(db, hasher);
    self.ast(db).stable_hash(db, hasher);
    self.diagnostics(db).stable_hash(db, hasher);
  }
}

impl Encodable<TypedownDatabase> for FileAstResult {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    self.handle(encoder.db).encode(encoder);
    self.project(encoder.db).encode(encoder);
    self.file(encoder.db).encode(encoder);
    self.ast(encoder.db).encode(encoder);
    self.diagnostics(encoder.db).encode(encoder);
  }
}

impl Decodable<TypedownDatabase> for FileAstResult {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    let handle = FileHandle::decode(decoder);
    let project = Project::decode(decoder);
    let file = File::decode(decoder);
    let ast = RedNode::decode(decoder);
    let diagnostics = Vec::decode(decoder);
    FileAstResult::new(decoder.db, handle, project, file, ast, diagnostics)
  }
}

#[query_derived]
pub struct SchemaAstResults {
  files: HashMap<PathBuf, FileAstResult>,
}

impl StableHash<TypedownDatabase> for SchemaAstResults {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.files(db).stable_hash(db, hasher);
  }
}

impl Encodable<TypedownDatabase> for SchemaAstResults {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    self.files(encoder.db).encode(encoder);
  }
}

impl Decodable<TypedownDatabase> for SchemaAstResults {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    let files = HashMap::decode(decoder);
    SchemaAstResults::new(decoder.db, files)
  }
}

#[query_derived]
pub struct TypecheckResult {
  diagnostics: Vec<Diagnostic>,
}

impl StableHash<TypedownDatabase> for TypecheckResult {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.diagnostics(db).stable_hash(db, hasher);
  }
}

impl Encodable<TypedownDatabase> for TypecheckResult {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    self.diagnostics(encoder.db).encode(encoder);
  }
}

impl Decodable<TypedownDatabase> for TypecheckResult {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    let diagnostics = Vec::decode(decoder);
    TypecheckResult::new(decoder.db, diagnostics)
  }
}

#[query_derived]
pub struct ResolveResult {
  diagnostics: Vec<Diagnostic>,
}

impl StableHash<TypedownDatabase> for ResolveResult {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.diagnostics(db).stable_hash(db, hasher);
  }
}

impl Encodable<TypedownDatabase> for ResolveResult {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    self.diagnostics(encoder.db).encode(encoder);
  }
}

impl Decodable<TypedownDatabase> for ResolveResult {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    let diagnostics = Vec::decode(decoder);
    ResolveResult::new(decoder.db, diagnostics)
  }
}

#[query_derived]
pub struct TypeResult {
  typ: Option<Box<dyn TdrTypeLike>>,
  diagnostics: Vec<Diagnostic>,
}

impl StableHash<TypedownDatabase> for TypeResult {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.typ(db).stable_hash(db, hasher);
    self.diagnostics(db).stable_hash(db, hasher);
  }
}

impl Encodable<TypedownDatabase> for TypeResult {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    self.typ(encoder.db).encode(encoder);
    self.diagnostics(encoder.db).encode(encoder);
  }
}

impl Decodable<TypedownDatabase> for TypeResult {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    let typ = Option::<Box<dyn TdrTypeLike>>::decode(decoder);
    let diagnostics = Vec::decode(decoder);
    TypeResult::new(decoder.db, typ, diagnostics)
  }
}

#[query_derived]
pub struct TypeMemberResult {
  member: Option<super::interned::TypeMember>,
  diagnostics: Vec<Diagnostic>,
}

impl StableHash<TypedownDatabase> for TypeMemberResult {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.member(db).stable_hash(db, hasher);
    self.diagnostics(db).stable_hash(db, hasher);
  }
}

impl Encodable<TypedownDatabase> for TypeMemberResult {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    self.member(encoder.db).encode(encoder);
    self.diagnostics(encoder.db).encode(encoder);
  }
}

impl Decodable<TypedownDatabase> for TypeMemberResult {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    let member = Option::decode(decoder);
    let diagnostics = Vec::decode(decoder);
    TypeMemberResult::new(decoder.db, member, diagnostics)
  }
}

#[query_derived]
pub struct InstResult {
  pub typ: Box<dyn TdrTypeLike>,
  pub diagnostics: Vec<Diagnostic>,
}

impl StableHash<TypedownDatabase> for InstResult {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.typ(db).stable_hash(db, hasher);
    self.diagnostics(db).stable_hash(db, hasher);
  }
}

impl Encodable<TypedownDatabase> for InstResult {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    self.typ(encoder.db).encode(encoder);
    self.diagnostics(encoder.db).encode(encoder);
  }
}

impl Decodable<TypedownDatabase> for InstResult {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    let typ = Box::<dyn TdrTypeLike>::decode(decoder);
    let diagnostics = Vec::decode(decoder);
    InstResult::new(decoder.db, typ, diagnostics)
  }
}

#[query_derived]
pub struct ResourceResult {
  pub value: Option<Box<dyn TdrObjectLike>>,
  pub diagnostics: Vec<Diagnostic>,
}

impl StableHash<TypedownDatabase> for ResourceResult {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.value(db).stable_hash(db, hasher);
    self.diagnostics(db).stable_hash(db, hasher);
  }
}

impl Encodable<TypedownDatabase> for ResourceResult {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    self.value(encoder.db).encode(encoder);
    self.diagnostics(encoder.db).encode(encoder);
  }
}

impl Decodable<TypedownDatabase> for ResourceResult {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    let value = Option::<Box<dyn TdrObjectLike>>::decode(decoder);
    let diagnostics = Vec::decode(decoder);
    ResourceResult::new(decoder.db, value, diagnostics)
  }
}
