use std::hash::Hasher;

use num_enum::TryFromPrimitive;
use typedown_macros::query_interned;

use crate::{
  Decodable, Decoder, Encodable, Encoder, QueryDatabase, StableHash, StableHasher, TypedownDatabase,
};

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

impl Encodable<TypedownDatabase> for FuncSignature {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    self.params(encoder.db).encode(encoder);
    self.ret(encoder.db).encode(encoder);
  }
}

impl Decodable<TypedownDatabase> for FuncSignature {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    let params = Vec::decode(decoder);
    let ret = Box::decode(decoder);
    FuncSignature::new(decoder.db, params, ret)
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

#[derive(TryFromPrimitive)]
#[repr(u8)]
enum LiteralValueTag {
  Str = 0,
  Bool = 1,
  Num = 2,
}

impl Encodable<TypedownDatabase> for LiteralValue {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    match self {
      LiteralValue::Str(val) => {
        encoder.emit_u8(LiteralValueTag::Str as u8);
        val.encode(encoder);
      }
      LiteralValue::Bool(val) => {
        encoder.emit_u8(LiteralValueTag::Bool as u8);
        val.encode(encoder);
      }
      LiteralValue::Num(val) => {
        encoder.emit_u8(LiteralValueTag::Num as u8);
        val.encode(encoder);
      }
    }
  }
}

impl Decodable<TypedownDatabase> for LiteralValue {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    let tag = decoder.read_u8();
    match LiteralValueTag::try_from(tag)
      .unwrap_or_else(|_| panic!("unknown LiteralValue tag {tag}"))
    {
      LiteralValueTag::Str => LiteralValue::Str(String::decode(decoder)),
      LiteralValueTag::Bool => LiteralValue::Bool(bool::decode(decoder)),
      LiteralValueTag::Num => LiteralValue::Num(String::decode(decoder)),
    }
  }
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

#[derive(TryFromPrimitive)]
#[repr(u8)]
enum MemberTypeTag {
  Simple = 0,
  Sum = 1,
  Literal = 2,
  Never = 3,
}

impl Encodable<TypedownDatabase> for MemberType {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    match self {
      MemberType::Simple(typ) => {
        encoder.emit_u8(MemberTypeTag::Simple as u8);
        typ.encode(encoder);
      }
      MemberType::Sum(members) => {
        encoder.emit_u8(MemberTypeTag::Sum as u8);
        members.encode(encoder);
      }
      MemberType::Literal(value) => {
        encoder.emit_u8(MemberTypeTag::Literal as u8);
        value.encode(encoder);
      }
      MemberType::Never => {
        encoder.emit_u8(MemberTypeTag::Never as u8);
      }
    }
  }
}

impl Decodable<TypedownDatabase> for MemberType {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    let tag = decoder.read_u8();
    match MemberTypeTag::try_from(tag).unwrap_or_else(|_| panic!("unknown MemberType tag {tag}")) {
      MemberTypeTag::Simple => MemberType::Simple(Box::decode(decoder)),
      MemberTypeTag::Sum => MemberType::Sum(Vec::decode(decoder)),
      MemberTypeTag::Literal => MemberType::Literal(LiteralValue::decode(decoder)),
      MemberTypeTag::Never => MemberType::Never,
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

impl Encodable<TypedownDatabase> for TypeMemberDescriptors {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    encoder.emit_u8(self.bits());
  }
}

impl Decodable<TypedownDatabase> for TypeMemberDescriptors {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    TypeMemberDescriptors::from_bits_truncate(decoder.read_u8())
  }
}

impl Encodable<TypedownDatabase> for TypeMember {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    self.typ(encoder.db).encode(encoder);
    self.descriptors(encoder.db).encode(encoder);
  }
}

impl Decodable<TypedownDatabase> for TypeMember {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    let typ = MemberType::decode(decoder);
    let descriptors = TypeMemberDescriptors::decode(decoder);
    TypeMember::new(decoder.db, typ, descriptors)
  }
}
