use std::hash::Hasher;

use typedown_macros::query_interned;

use crate::{QueryDatabase, StableHash, StableHasher, TypedownDatabase};

use super::TdrTypeLike;

#[query_interned]
pub struct FuncSignature {
  pub params: Vec<Box<dyn TdrTypeLike>>,
  pub ret: Box<dyn TdrTypeLike>,
}

impl StableHash<TypedownDatabase> for FuncSignature {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.params(db).stable_hash(db, hasher);
    self.ret(db).stable_hash(db, hasher);
  }
}

bitflags::bitflags! {
  #[derive(Clone, Copy, PartialEq, Eq, Hash)]
  pub struct TypeMemberDescriptors: u8 {
    const OPTIONAL = 0b0000_0001;
  }
}

impl<DB: QueryDatabase> StableHash<DB> for TypeMemberDescriptors {
  fn stable_hash(&self, _db: &DB, hasher: &mut StableHasher) {
    hasher.write_u8(self.bits());
  }
}

/// The type of a type member field
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum MemberType {
  /// A reference to a named type (e.g. `string`, `list[number]`)
  Simple(Box<dyn TdrTypeLike>),
  /// A union or enum type: each arm is itself a `TypeMember` (a type ref)
  Sum(Vec<TypeMember>),
  /// A literal value constraint (e.g. `"foo"`, `42`, `true`)
  Literal(LiteralValue),
  /// The bottom type: no value can be assigned to this field
  Never,
}

/// A concrete literal value used in literal constraints
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LiteralValue {
  Str(String),
  Bool(bool),
  // f64 cannot be hashed so we store in string
  Num(String),
}

impl<DB: QueryDatabase> StableHash<DB> for LiteralValue {
  fn stable_hash(&self, db: &DB, hasher: &mut StableHasher) {
    std::mem::discriminant(self).stable_hash(db, hasher);
    match self {
      LiteralValue::Str(value) => value.stable_hash(db, hasher),
      LiteralValue::Bool(value) => value.stable_hash(db, hasher),
      LiteralValue::Num(value) => value.stable_hash(db, hasher),
    }
  }
}

impl StableHash<TypedownDatabase> for MemberType {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    std::mem::discriminant(self).stable_hash(db, hasher);
    match self {
      MemberType::Simple(typ) => typ.stable_hash(db, hasher),
      MemberType::Sum(members) => members.stable_hash(db, hasher),
      MemberType::Literal(value) => value.stable_hash(db, hasher),
      MemberType::Never => {}
    }
  }
}

#[query_interned]
pub struct TypeMember {
  pub typ: MemberType,
  pub descriptors: TypeMemberDescriptors,
}

impl StableHash<TypedownDatabase> for TypeMember {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.typ(db).stable_hash(db, hasher);
    self.descriptors(db).stable_hash(db, hasher);
  }
}
