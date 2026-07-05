use std::collections::HashMap;

use num_enum::TryFromPrimitive;
use typedown_macros::query_derived;

use crate::types::{File, Project};
use typedown_incremental::{Decodable, Decoder, Encodable, Encoder, StableHash, StableHasher};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SymbolKind {
  UserDefinedSchema(Project, File),
  UserDefinedResource(Project, File),
  BuiltinSchema(BuiltinSchemaKind),
  BuiltinMacro(BuiltinMacroKind),
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
enum SymbolKindTag {
  UserDefinedSchema = 0,
  UserDefinedResource = 1,
  BuiltinSchema = 2,
  BuiltinMacro = 3,
}

impl SymbolKind {
  pub fn is_schema(&self) -> bool {
    matches!(
      self,
      SymbolKind::UserDefinedSchema(_, _) | SymbolKind::BuiltinSchema(_)
    )
  }

  pub fn is_resource(&self) -> bool {
    matches!(self, SymbolKind::UserDefinedResource(_, _))
  }

  pub fn is_user_defined(&self) -> bool {
    matches!(
      self,
      SymbolKind::UserDefinedSchema(_, _) | SymbolKind::UserDefinedResource(_, _)
    )
  }

  pub fn is_builtin(&self) -> bool {
    matches!(self, SymbolKind::BuiltinSchema(_))
  }
}

impl StableHash for SymbolKind {
  fn stable_hash<DB: ::typedown_incremental::QueryDatabase + ?Sized>(
    &self,
    db: &DB,
    hasher: &mut StableHasher,
  ) {
    std::mem::discriminant(self).stable_hash(db, hasher);
    match self {
      SymbolKind::UserDefinedSchema(project, file)
      | SymbolKind::UserDefinedResource(project, file) => {
        project.stable_hash(db, hasher);
        file.stable_hash(db, hasher);
      }
      SymbolKind::BuiltinSchema(kind) => kind.stable_hash(db, hasher),
      SymbolKind::BuiltinMacro(kind) => kind.stable_hash(db, hasher),
    }
  }
}

impl Encodable for SymbolKind {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    match self {
      SymbolKind::UserDefinedSchema(project, file) => {
        encoder.emit_u8(SymbolKindTag::UserDefinedSchema as u8);
        project.encode(encoder);
        file.encode(encoder);
      }
      SymbolKind::UserDefinedResource(project, file) => {
        encoder.emit_u8(SymbolKindTag::UserDefinedResource as u8);
        project.encode(encoder);
        file.encode(encoder);
      }
      SymbolKind::BuiltinSchema(kind) => {
        encoder.emit_u8(SymbolKindTag::BuiltinSchema as u8);
        kind.encode(encoder);
      }
      SymbolKind::BuiltinMacro(kind) => {
        encoder.emit_u8(SymbolKindTag::BuiltinMacro as u8);
        kind.encode(encoder);
      }
    }
  }
}

impl Decodable for SymbolKind {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    let tag = decoder.read_u8();
    match SymbolKindTag::try_from(tag).unwrap_or_else(|_| panic!("unknown SymbolKind tag {tag}")) {
      SymbolKindTag::UserDefinedSchema => {
        SymbolKind::UserDefinedSchema(Project::decode(decoder), File::decode(decoder))
      }
      SymbolKindTag::UserDefinedResource => {
        SymbolKind::UserDefinedResource(Project::decode(decoder), File::decode(decoder))
      }
      SymbolKindTag::BuiltinSchema => SymbolKind::BuiltinSchema(BuiltinSchemaKind::decode(decoder)),
      SymbolKindTag::BuiltinMacro => SymbolKind::BuiltinMacro(BuiltinMacroKind::decode(decoder)),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum BuiltinMacroKind {
  Fref = 0,
}

impl StableHash for BuiltinMacroKind {
  fn stable_hash<DB: ::typedown_incremental::QueryDatabase + ?Sized>(
    &self,
    db: &DB,
    hasher: &mut StableHasher,
  ) {
    std::mem::discriminant(self).stable_hash(db, hasher);
  }
}

impl Encodable for BuiltinMacroKind {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_u8(*self as u8);
  }
}

impl Decodable for BuiltinMacroKind {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    let tag = decoder.read_u8();
    BuiltinMacroKind::try_from(tag).unwrap_or_else(|_| panic!("unknown BuiltinMacroKind tag {tag}"))
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum BuiltinSchemaKind {
  TypeType = 0,
  Schema = 1,
  Str = 2,
  Num = 3,
  Bool = 4,
  Date = 5,
  DateTime = 6,
  Time = 7,
  List = 8,
  Dict = 9,
  Math = 10,
}

impl StableHash for BuiltinSchemaKind {
  fn stable_hash<DB: ::typedown_incremental::QueryDatabase + ?Sized>(
    &self,
    db: &DB,
    hasher: &mut StableHasher,
  ) {
    std::mem::discriminant(self).stable_hash(db, hasher);
  }
}

impl Encodable for BuiltinSchemaKind {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_u8(*self as u8);
  }
}

impl Decodable for BuiltinSchemaKind {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    let tag = decoder.read_u8();
    BuiltinSchemaKind::try_from(tag)
      .unwrap_or_else(|_| panic!("unknown BuiltinSchemaKind tag {tag}"))
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScopeKind {
  Builtin,
  Project(Project),
  File(Project, File),
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
enum ScopeKindTag {
  Builtin = 0,
  Project = 1,
  File = 2,
}

impl StableHash for ScopeKind {
  fn stable_hash<DB: ::typedown_incremental::QueryDatabase + ?Sized>(
    &self,
    db: &DB,
    hasher: &mut StableHasher,
  ) {
    std::mem::discriminant(self).stable_hash(db, hasher);
    match self {
      ScopeKind::Builtin => {}
      ScopeKind::Project(project) => project.stable_hash(db, hasher),
      ScopeKind::File(project, file) => {
        project.stable_hash(db, hasher);
        file.stable_hash(db, hasher);
      }
    }
  }
}

impl Encodable for ScopeKind {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    match self {
      ScopeKind::Builtin => encoder.emit_u8(ScopeKindTag::Builtin as u8),
      ScopeKind::Project(project) => {
        encoder.emit_u8(ScopeKindTag::Project as u8);
        project.encode(encoder);
      }
      ScopeKind::File(project, file) => {
        encoder.emit_u8(ScopeKindTag::File as u8);
        project.encode(encoder);
        file.encode(encoder);
      }
    }
  }
}

impl Decodable for ScopeKind {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    let tag = decoder.read_u8();
    match ScopeKindTag::try_from(tag).unwrap_or_else(|_| panic!("unknown ScopeKind tag {tag}")) {
      ScopeKindTag::Builtin => ScopeKind::Builtin,
      ScopeKindTag::Project => ScopeKind::Project(Project::decode(decoder)),
      ScopeKindTag::File => ScopeKind::File(Project::decode(decoder), File::decode(decoder)),
    }
  }
}

#[query_derived]
pub struct Scope {
  #[id]
  kind: ScopeKind,
}

impl Scope {
  pub fn builtin_scope(db: &(impl typedown_incremental::QueryDatabase + ?Sized)) -> Self {
    Self::new(db, ScopeKind::Builtin)
  }

  pub fn project_scope(
    db: &(impl typedown_incremental::QueryDatabase + ?Sized),
    project: Project,
  ) -> Self {
    Self::new(db, ScopeKind::Project(project))
  }

  pub fn file_scope(
    db: &(impl typedown_incremental::QueryDatabase + ?Sized),
    project: Project,
    file: File,
  ) -> Self {
    Self::new(db, ScopeKind::File(project, file))
  }
}

#[query_derived]
pub struct Symbol {
  #[id]
  kind: SymbolKind,
  name: String,
}

#[query_derived]
pub struct ProjectSchemaResult {
  members: HashMap<String, Symbol>,
}

#[query_derived]
pub struct MembersResult {
  members: HashMap<String, Symbol>,
}
