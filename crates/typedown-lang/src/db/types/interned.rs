use std::hash::Hasher;

use strum::FromRepr;
use typedown_macros::query_interned;

use typedown_incremental::{
  Decodable, Decoder, Encodable, Encoder, QueryDatabase, StableHash, StableHasher,
};

use super::TdrTypeEnum;

#[query_interned]
pub struct FuncSignature {
  pub params: Vec<TdrTypeEnum>,
  pub ret: TdrTypeEnum,
}

bitflags::bitflags! {
  #[derive(Clone, Copy, PartialEq, Eq, Hash)]
  pub struct TypeMemberDescriptors: u8 {
    const OPTIONAL = 0b0000_0001;
  }
}

impl StableHash for TypeMemberDescriptors {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, _db: &DB, hasher: &mut StableHasher) {
    hasher.write_u8(self.bits());
  }
}

/// The type of a type member field
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum MemberType {
  /// A reference to a named type (e.g. `string`, `list[number]`)
  Simple(TdrTypeEnum),
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

#[derive(FromRepr)]
#[repr(u8)]
enum LiteralValueTag {
  Str = 0,
  Bool = 1,
  Num = 2,
}

impl Encodable for LiteralValue {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
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

impl Decodable for LiteralValue {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    let tag = decoder.read_u8();
    match LiteralValueTag::from_repr(tag)
      .unwrap_or_else(|| panic!("unknown LiteralValue tag {tag}"))
    {
      LiteralValueTag::Str => LiteralValue::Str(String::decode(decoder)),
      LiteralValueTag::Bool => LiteralValue::Bool(bool::decode(decoder)),
      LiteralValueTag::Num => LiteralValue::Num(String::decode(decoder)),
    }
  }
}

impl StableHash for LiteralValue {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    std::mem::discriminant(self).stable_hash(db, hasher);
    match self {
      LiteralValue::Str(value) => value.stable_hash(db, hasher),
      LiteralValue::Bool(value) => value.stable_hash(db, hasher),
      LiteralValue::Num(value) => value.stable_hash(db, hasher),
    }
  }
}

#[derive(FromRepr)]
#[repr(u8)]
enum MemberTypeTag {
  Simple = 0,
  Sum = 1,
  Literal = 2,
  Never = 3,
}

impl Encodable for MemberType {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
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

impl Decodable for MemberType {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    let tag = decoder.read_u8();
    match MemberTypeTag::from_repr(tag).unwrap_or_else(|| panic!("unknown MemberType tag {tag}")) {
      MemberTypeTag::Simple => MemberType::Simple(TdrTypeEnum::decode(decoder)),
      MemberTypeTag::Sum => MemberType::Sum(Vec::decode(decoder)),
      MemberTypeTag::Literal => MemberType::Literal(LiteralValue::decode(decoder)),
      MemberTypeTag::Never => MemberType::Never,
    }
  }
}

impl StableHash for MemberType {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
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

impl Encodable for TypeMemberDescriptors {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_u8(self.bits());
  }
}

impl Decodable for TypeMemberDescriptors {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    TypeMemberDescriptors::from_bits_truncate(decoder.read_u8())
  }
}
