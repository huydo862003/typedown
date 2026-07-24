mod base;
mod blob;
mod bool;
mod datetime;
mod dict;
mod func;
mod list;
mod math;
mod native_fn;
mod num;
mod product;
mod schema;
mod schema_property;
mod str;

use std::hash::{Hash, Hasher};

use strum::FromRepr;

use tdr_incremental::{Decodable, Decoder, Encodable, Encoder, FieldDecodable, FieldEncodable};

pub use base::*;
pub use blob::*;
pub use bool::*;
pub use datetime::*;
pub use dict::*;
pub use func::*;
pub use list::*;
pub use math::*;
pub use native_fn::*;
pub use num::*;
pub use product::*;
pub use schema::*;
pub use schema_property::*;
pub use str::*;

use ambassador::Delegate;
use derive_more::From;
use enum_as_inner::EnumAsInner;

use tdr_incremental::Id;

/// Use this instead of dyn
/// The primitive types are fixed anyways
#[derive(Clone, From, Delegate, EnumAsInner)]
#[delegate(TdrObjectLike)]
#[delegate(TdrTypeLike)]
pub enum TdrTypeEnum {
  TdrTypeType(TdrTypeType),
  TdrObjectType(TdrObjectType),
  TdrBoolType(TdrBoolType),
  TdrStrType(TdrStrType),
  TdrNumType(TdrNumType),
  TdrMathType(TdrMathType),
  TdrFuncType(TdrFuncType),
  TdrListType(TdrListType),
  TdrDictType(TdrDictType),
  TdrDateTimeType(TdrDateTimeType),
  TdrDateType(TdrDateType),
  TdrTimeType(TdrTimeType),
  TdrSchemaType(TdrSchemaType),
  TdrSchemaPropertyType(TdrSchemaPropertyType),
  TdrProductType(TdrProductType),
  TdrBlobType(TdrBlobType),
}

/// Use this instead of dyn
/// The primitive object kinds are fixed anyways
#[derive(Clone, From, Delegate, EnumAsInner)]
#[delegate(TdrObjectLike)]
pub enum TdrObjectEnum {
  // Types are objects
  TdrTypeType(TdrTypeType),
  TdrObjectType(TdrObjectType),
  TdrBoolType(TdrBoolType),
  TdrStrType(TdrStrType),
  TdrNumType(TdrNumType),
  TdrMathType(TdrMathType),
  TdrFuncType(TdrFuncType),
  TdrListType(TdrListType),
  TdrDictType(TdrDictType),
  TdrDateTimeType(TdrDateTimeType),
  TdrDateType(TdrDateType),
  TdrTimeType(TdrTimeType),
  TdrSchemaType(TdrSchemaType),
  TdrSchemaPropertyType(TdrSchemaPropertyType),
  TdrProductType(TdrProductType),
  TdrBlobType(TdrBlobType),
  // Objects
  TdrBoolObj(TdrBoolObj),
  TdrStrObj(TdrStrObj),
  TdrNumObj(TdrNumObj),
  TdrMathObj(TdrMathObj),
  TdrFuncObj(TdrFuncObj),
  TdrListObj(TdrListObj),
  TdrDictObj(TdrDictObj),
  TdrDateTimeObj(TdrDateTimeObj),
  TdrDateObj(TdrDateObj),
  TdrTimeObj(TdrTimeObj),
  TdrProductObj(TdrProductObj),
  TdrBlobObj(TdrBlobObj),
}

impl Id for TdrTypeEnum {
  fn as_id(&self) -> (usize, usize) {
    match self {
      TdrTypeEnum::TdrTypeType(v) => v.as_id(),
      TdrTypeEnum::TdrObjectType(v) => v.as_id(),
      TdrTypeEnum::TdrBoolType(v) => v.as_id(),
      TdrTypeEnum::TdrStrType(v) => v.as_id(),
      TdrTypeEnum::TdrNumType(v) => v.as_id(),
      TdrTypeEnum::TdrMathType(v) => v.as_id(),
      TdrTypeEnum::TdrFuncType(v) => v.as_id(),
      TdrTypeEnum::TdrListType(v) => v.as_id(),
      TdrTypeEnum::TdrDictType(v) => v.as_id(),
      TdrTypeEnum::TdrDateTimeType(v) => v.as_id(),
      TdrTypeEnum::TdrDateType(v) => v.as_id(),
      TdrTypeEnum::TdrTimeType(v) => v.as_id(),
      TdrTypeEnum::TdrSchemaType(v) => v.as_id(),
      TdrTypeEnum::TdrSchemaPropertyType(v) => v.as_id(),
      TdrTypeEnum::TdrProductType(v) => v.as_id(),
      TdrTypeEnum::TdrBlobType(v) => v.as_id(),
    }
  }
}

impl Id for TdrObjectEnum {
  fn as_id(&self) -> (usize, usize) {
    match self {
      TdrObjectEnum::TdrTypeType(v) => v.as_id(),
      TdrObjectEnum::TdrObjectType(v) => v.as_id(),
      TdrObjectEnum::TdrBoolType(v) => v.as_id(),
      TdrObjectEnum::TdrStrType(v) => v.as_id(),
      TdrObjectEnum::TdrNumType(v) => v.as_id(),
      TdrObjectEnum::TdrMathType(v) => v.as_id(),
      TdrObjectEnum::TdrFuncType(v) => v.as_id(),
      TdrObjectEnum::TdrListType(v) => v.as_id(),
      TdrObjectEnum::TdrDictType(v) => v.as_id(),
      TdrObjectEnum::TdrDateTimeType(v) => v.as_id(),
      TdrObjectEnum::TdrDateType(v) => v.as_id(),
      TdrObjectEnum::TdrTimeType(v) => v.as_id(),
      TdrObjectEnum::TdrSchemaType(v) => v.as_id(),
      TdrObjectEnum::TdrSchemaPropertyType(v) => v.as_id(),
      TdrObjectEnum::TdrProductType(v) => v.as_id(),
      TdrObjectEnum::TdrBlobType(v) => v.as_id(),
      TdrObjectEnum::TdrBoolObj(v) => v.as_id(),
      TdrObjectEnum::TdrStrObj(v) => v.as_id(),
      TdrObjectEnum::TdrNumObj(v) => v.as_id(),
      TdrObjectEnum::TdrMathObj(v) => v.as_id(),
      TdrObjectEnum::TdrFuncObj(v) => v.as_id(),
      TdrObjectEnum::TdrListObj(v) => v.as_id(),
      TdrObjectEnum::TdrDictObj(v) => v.as_id(),
      TdrObjectEnum::TdrDateTimeObj(v) => v.as_id(),
      TdrObjectEnum::TdrDateObj(v) => v.as_id(),
      TdrObjectEnum::TdrTimeObj(v) => v.as_id(),
      TdrObjectEnum::TdrProductObj(v) => v.as_id(),
      TdrObjectEnum::TdrBlobObj(v) => v.as_id(),
    }
  }
}

impl From<TdrTypeEnum> for TdrObjectEnum {
  fn from(ty: TdrTypeEnum) -> Self {
    match ty {
      TdrTypeEnum::TdrTypeType(v) => TdrObjectEnum::TdrTypeType(v),
      TdrTypeEnum::TdrObjectType(v) => TdrObjectEnum::TdrObjectType(v),
      TdrTypeEnum::TdrBoolType(v) => TdrObjectEnum::TdrBoolType(v),
      TdrTypeEnum::TdrStrType(v) => TdrObjectEnum::TdrStrType(v),
      TdrTypeEnum::TdrNumType(v) => TdrObjectEnum::TdrNumType(v),
      TdrTypeEnum::TdrMathType(v) => TdrObjectEnum::TdrMathType(v),
      TdrTypeEnum::TdrFuncType(v) => TdrObjectEnum::TdrFuncType(v),
      TdrTypeEnum::TdrListType(v) => TdrObjectEnum::TdrListType(v),
      TdrTypeEnum::TdrDictType(v) => TdrObjectEnum::TdrDictType(v),
      TdrTypeEnum::TdrDateTimeType(v) => TdrObjectEnum::TdrDateTimeType(v),
      TdrTypeEnum::TdrDateType(v) => TdrObjectEnum::TdrDateType(v),
      TdrTypeEnum::TdrTimeType(v) => TdrObjectEnum::TdrTimeType(v),
      TdrTypeEnum::TdrSchemaType(v) => TdrObjectEnum::TdrSchemaType(v),
      TdrTypeEnum::TdrSchemaPropertyType(v) => TdrObjectEnum::TdrSchemaPropertyType(v),
      TdrTypeEnum::TdrProductType(v) => TdrObjectEnum::TdrProductType(v),
      TdrTypeEnum::TdrBlobType(v) => TdrObjectEnum::TdrBlobType(v),
    }
  }
}

impl TdrObjectEnum {
  pub fn as_type(self) -> Option<TdrTypeEnum> {
    match self {
      TdrObjectEnum::TdrTypeType(v) => Some(TdrTypeEnum::TdrTypeType(v)),
      TdrObjectEnum::TdrObjectType(v) => Some(TdrTypeEnum::TdrObjectType(v)),
      TdrObjectEnum::TdrBoolType(v) => Some(TdrTypeEnum::TdrBoolType(v)),
      TdrObjectEnum::TdrStrType(v) => Some(TdrTypeEnum::TdrStrType(v)),
      TdrObjectEnum::TdrNumType(v) => Some(TdrTypeEnum::TdrNumType(v)),
      TdrObjectEnum::TdrMathType(v) => Some(TdrTypeEnum::TdrMathType(v)),
      TdrObjectEnum::TdrFuncType(v) => Some(TdrTypeEnum::TdrFuncType(v)),
      TdrObjectEnum::TdrListType(v) => Some(TdrTypeEnum::TdrListType(v)),
      TdrObjectEnum::TdrDictType(v) => Some(TdrTypeEnum::TdrDictType(v)),
      TdrObjectEnum::TdrDateTimeType(v) => Some(TdrTypeEnum::TdrDateTimeType(v)),
      TdrObjectEnum::TdrDateType(v) => Some(TdrTypeEnum::TdrDateType(v)),
      TdrObjectEnum::TdrTimeType(v) => Some(TdrTypeEnum::TdrTimeType(v)),
      TdrObjectEnum::TdrSchemaType(v) => Some(TdrTypeEnum::TdrSchemaType(v)),
      TdrObjectEnum::TdrSchemaPropertyType(v) => Some(TdrTypeEnum::TdrSchemaPropertyType(v)),
      TdrObjectEnum::TdrProductType(v) => Some(TdrTypeEnum::TdrProductType(v)),
      TdrObjectEnum::TdrBlobType(v) => Some(TdrTypeEnum::TdrBlobType(v)),
      _ => None,
    }
  }
}

impl PartialEq for TdrTypeEnum {
  fn eq(&self, other: &Self) -> bool {
    self.as_id() == other.as_id()
  }
}
impl Eq for TdrTypeEnum {}

impl Hash for TdrTypeEnum {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.as_id().hash(state);
  }
}

impl PartialEq for TdrObjectEnum {
  fn eq(&self, other: &Self) -> bool {
    self.as_id() == other.as_id()
  }
}
impl Eq for TdrObjectEnum {}

impl Hash for TdrObjectEnum {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.as_id().hash(state);
  }
}

impl tdr_incremental::StableHash for TdrTypeEnum {
  fn stable_hash<DB: tdr_incremental::QueryDatabase + ?Sized>(
    &self,
    db: &DB,
    hasher: &mut tdr_incremental::StableHasher,
  ) {
    match self {
      TdrTypeEnum::TdrTypeType(v) => v.stable_hash(db, hasher),
      TdrTypeEnum::TdrObjectType(v) => v.stable_hash(db, hasher),
      TdrTypeEnum::TdrBoolType(v) => v.stable_hash(db, hasher),
      TdrTypeEnum::TdrStrType(v) => v.stable_hash(db, hasher),
      TdrTypeEnum::TdrNumType(v) => v.stable_hash(db, hasher),
      TdrTypeEnum::TdrMathType(v) => v.stable_hash(db, hasher),
      TdrTypeEnum::TdrFuncType(v) => v.stable_hash(db, hasher),
      TdrTypeEnum::TdrListType(v) => v.stable_hash(db, hasher),
      TdrTypeEnum::TdrDictType(v) => v.stable_hash(db, hasher),
      TdrTypeEnum::TdrDateTimeType(v) => v.stable_hash(db, hasher),
      TdrTypeEnum::TdrDateType(v) => v.stable_hash(db, hasher),
      TdrTypeEnum::TdrTimeType(v) => v.stable_hash(db, hasher),
      TdrTypeEnum::TdrSchemaType(v) => v.stable_hash(db, hasher),
      TdrTypeEnum::TdrSchemaPropertyType(v) => v.stable_hash(db, hasher),
      TdrTypeEnum::TdrProductType(v) => v.stable_hash(db, hasher),
      TdrTypeEnum::TdrBlobType(v) => v.stable_hash(db, hasher),
    }
  }
}

impl tdr_incremental::StableHash for TdrObjectEnum {
  fn stable_hash<DB: tdr_incremental::QueryDatabase + ?Sized>(
    &self,
    db: &DB,
    hasher: &mut tdr_incremental::StableHasher,
  ) {
    match self {
      TdrObjectEnum::TdrTypeType(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrObjectType(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrBoolType(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrStrType(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrNumType(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrMathType(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrFuncType(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrListType(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrDictType(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrDateTimeType(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrDateType(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrTimeType(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrSchemaType(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrSchemaPropertyType(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrProductType(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrBlobType(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrBoolObj(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrStrObj(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrNumObj(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrMathObj(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrFuncObj(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrListObj(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrDictObj(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrDateTimeObj(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrDateObj(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrTimeObj(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrProductObj(v) => v.stable_hash(db, hasher),
      TdrObjectEnum::TdrBlobObj(v) => v.stable_hash(db, hasher),
    }
  }
}

#[derive(FromRepr)]
#[repr(u8)]
pub enum TdrTypeKind {
  Type = 0,
  Object = 1,
  Str = 2,
  Bool = 3,
  Num = 4,
  Math = 5,
  List = 6,
  Dict = 7,
  Func = 8,
  Product = 9,
  Schema = 10,
  SchemaProperty = 11,
  DateTime = 12,
  Date = 13,
  Time = 14,
  Blob = 15,
}

#[derive(FromRepr)]
#[repr(u8)]
pub enum TdrObjectKind {
  // Types (also objects)
  Type = 0,
  Object = 1,
  Str = 2,
  Bool = 3,
  Num = 4,
  Math = 5,
  List = 6,
  Dict = 7,
  Func = 8,
  Product = 9,
  Schema = 10,
  SchemaProperty = 11,
  DateTime = 12,
  Date = 13,
  Time = 14,
  Blob = 15,
  // Object-only
  StrObj = 128,
  BoolObj = 129,
  NumObj = 130,
  MathObj = 131,
  ListObj = 132,
  DictObj = 133,
  FuncObj = 134,
  ProductObj = 135,
  DateTimeObj = 136,
  DateObj = 137,
  TimeObj = 138,
  BlobObj = 139,
}

// TdrTypeEnum
impl Encodable for TdrTypeEnum {
  fn encode(&self, buf: &mut Vec<u8>, encoder: &mut Encoder) {
    match self {
      TdrTypeEnum::TdrTypeType(v) => {
        encoder.emit_u8(buf, TdrTypeKind::Type as u8);
        v.encode_field(buf, encoder);
      }
      TdrTypeEnum::TdrObjectType(v) => {
        encoder.emit_u8(buf, TdrTypeKind::Object as u8);
        v.encode_field(buf, encoder);
      }
      TdrTypeEnum::TdrStrType(v) => {
        encoder.emit_u8(buf, TdrTypeKind::Str as u8);
        v.encode_field(buf, encoder);
      }
      TdrTypeEnum::TdrBoolType(v) => {
        encoder.emit_u8(buf, TdrTypeKind::Bool as u8);
        v.encode_field(buf, encoder);
      }
      TdrTypeEnum::TdrNumType(v) => {
        encoder.emit_u8(buf, TdrTypeKind::Num as u8);
        v.encode_field(buf, encoder);
      }
      TdrTypeEnum::TdrMathType(v) => {
        encoder.emit_u8(buf, TdrTypeKind::Math as u8);
        v.encode_field(buf, encoder);
      }
      TdrTypeEnum::TdrListType(v) => {
        encoder.emit_u8(buf, TdrTypeKind::List as u8);
        v.encode_field(buf, encoder);
      }
      TdrTypeEnum::TdrDictType(v) => {
        encoder.emit_u8(buf, TdrTypeKind::Dict as u8);
        v.encode_field(buf, encoder);
      }
      TdrTypeEnum::TdrFuncType(v) => {
        encoder.emit_u8(buf, TdrTypeKind::Func as u8);
        v.encode_field(buf, encoder);
      }
      TdrTypeEnum::TdrProductType(v) => {
        encoder.emit_u8(buf, TdrTypeKind::Product as u8);
        v.encode_field(buf, encoder);
      }
      TdrTypeEnum::TdrSchemaType(v) => {
        encoder.emit_u8(buf, TdrTypeKind::Schema as u8);
        v.encode_field(buf, encoder);
      }
      TdrTypeEnum::TdrSchemaPropertyType(v) => {
        encoder.emit_u8(buf, TdrTypeKind::SchemaProperty as u8);
        v.encode_field(buf, encoder);
      }
      TdrTypeEnum::TdrDateTimeType(v) => {
        encoder.emit_u8(buf, TdrTypeKind::DateTime as u8);
        v.encode_field(buf, encoder);
      }
      TdrTypeEnum::TdrDateType(v) => {
        encoder.emit_u8(buf, TdrTypeKind::Date as u8);
        v.encode_field(buf, encoder);
      }
      TdrTypeEnum::TdrTimeType(v) => {
        encoder.emit_u8(buf, TdrTypeKind::Time as u8);
        v.encode_field(buf, encoder);
      }
      TdrTypeEnum::TdrBlobType(v) => {
        encoder.emit_u8(buf, TdrTypeKind::Blob as u8);
        v.encode_field(buf, encoder);
      }
    }
  }
}

impl Decodable for TdrTypeEnum {
  fn decode(data: &mut &[u8], decoder: &Decoder) -> Self {
    let tag = decoder.read_u8(data);
    match TdrTypeKind::from_repr(tag).expect("unknown TdrTypeKind tag") {
      TdrTypeKind::Type => TdrTypeType::decode_field(data, decoder).into(),
      TdrTypeKind::Object => TdrObjectType::decode_field(data, decoder).into(),
      TdrTypeKind::Str => TdrStrType::decode_field(data, decoder).into(),
      TdrTypeKind::Bool => TdrBoolType::decode_field(data, decoder).into(),
      TdrTypeKind::Num => TdrNumType::decode_field(data, decoder).into(),
      TdrTypeKind::Math => TdrMathType::decode_field(data, decoder).into(),
      TdrTypeKind::List => TdrListType::decode_field(data, decoder).into(),
      TdrTypeKind::Dict => TdrDictType::decode_field(data, decoder).into(),
      TdrTypeKind::Func => TdrFuncType::decode_field(data, decoder).into(),
      TdrTypeKind::Product => TdrProductType::decode_field(data, decoder).into(),
      TdrTypeKind::Schema => TdrSchemaType::decode_field(data, decoder).into(),
      TdrTypeKind::SchemaProperty => TdrSchemaPropertyType::decode_field(data, decoder).into(),
      TdrTypeKind::DateTime => TdrDateTimeType::decode_field(data, decoder).into(),
      TdrTypeKind::Date => TdrDateType::decode_field(data, decoder).into(),
      TdrTypeKind::Time => TdrTimeType::decode_field(data, decoder).into(),
      TdrTypeKind::Blob => TdrBlobType::decode_field(data, decoder).into(),
    }
  }
}

// TdrObjectEnum
impl Encodable for TdrObjectEnum {
  fn encode(&self, buf: &mut Vec<u8>, encoder: &mut Encoder) {
    match self {
      // Types
      TdrObjectEnum::TdrTypeType(v) => {
        encoder.emit_u8(buf, TdrObjectKind::Type as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrObjectType(v) => {
        encoder.emit_u8(buf, TdrObjectKind::Object as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrStrType(v) => {
        encoder.emit_u8(buf, TdrObjectKind::Str as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrBoolType(v) => {
        encoder.emit_u8(buf, TdrObjectKind::Bool as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrNumType(v) => {
        encoder.emit_u8(buf, TdrObjectKind::Num as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrMathType(v) => {
        encoder.emit_u8(buf, TdrObjectKind::Math as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrListType(v) => {
        encoder.emit_u8(buf, TdrObjectKind::List as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrDictType(v) => {
        encoder.emit_u8(buf, TdrObjectKind::Dict as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrFuncType(v) => {
        encoder.emit_u8(buf, TdrObjectKind::Func as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrProductType(v) => {
        encoder.emit_u8(buf, TdrObjectKind::Product as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrSchemaType(v) => {
        encoder.emit_u8(buf, TdrObjectKind::Schema as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrSchemaPropertyType(v) => {
        encoder.emit_u8(buf, TdrObjectKind::SchemaProperty as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrDateTimeType(v) => {
        encoder.emit_u8(buf, TdrObjectKind::DateTime as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrDateType(v) => {
        encoder.emit_u8(buf, TdrObjectKind::Date as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrTimeType(v) => {
        encoder.emit_u8(buf, TdrObjectKind::Time as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrBlobType(v) => {
        encoder.emit_u8(buf, TdrObjectKind::Blob as u8);
        v.encode_field(buf, encoder);
      }
      // Objects
      TdrObjectEnum::TdrStrObj(v) => {
        encoder.emit_u8(buf, TdrObjectKind::StrObj as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrBoolObj(v) => {
        encoder.emit_u8(buf, TdrObjectKind::BoolObj as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrNumObj(v) => {
        encoder.emit_u8(buf, TdrObjectKind::NumObj as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrMathObj(v) => {
        encoder.emit_u8(buf, TdrObjectKind::MathObj as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrListObj(v) => {
        encoder.emit_u8(buf, TdrObjectKind::ListObj as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrDictObj(v) => {
        encoder.emit_u8(buf, TdrObjectKind::DictObj as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrFuncObj(v) => {
        encoder.emit_u8(buf, TdrObjectKind::FuncObj as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrProductObj(v) => {
        encoder.emit_u8(buf, TdrObjectKind::ProductObj as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrDateTimeObj(v) => {
        encoder.emit_u8(buf, TdrObjectKind::DateTimeObj as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrDateObj(v) => {
        encoder.emit_u8(buf, TdrObjectKind::DateObj as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrTimeObj(v) => {
        encoder.emit_u8(buf, TdrObjectKind::TimeObj as u8);
        v.encode_field(buf, encoder);
      }
      TdrObjectEnum::TdrBlobObj(v) => {
        encoder.emit_u8(buf, TdrObjectKind::BlobObj as u8);
        v.encode_field(buf, encoder);
      }
    }
  }
}

impl Decodable for TdrObjectEnum {
  fn decode(data: &mut &[u8], decoder: &Decoder) -> Self {
    let tag = decoder.read_u8(data);
    match TdrObjectKind::from_repr(tag).expect("unknown TdrObjectKind tag") {
      TdrObjectKind::Type => TdrTypeType::decode_field(data, decoder).into(),
      TdrObjectKind::Object => TdrObjectType::decode_field(data, decoder).into(),
      TdrObjectKind::Str => TdrStrType::decode_field(data, decoder).into(),
      TdrObjectKind::Bool => TdrBoolType::decode_field(data, decoder).into(),
      TdrObjectKind::Num => TdrNumType::decode_field(data, decoder).into(),
      TdrObjectKind::Math => TdrMathType::decode_field(data, decoder).into(),
      TdrObjectKind::List => TdrListType::decode_field(data, decoder).into(),
      TdrObjectKind::Dict => TdrDictType::decode_field(data, decoder).into(),
      TdrObjectKind::Func => TdrFuncType::decode_field(data, decoder).into(),
      TdrObjectKind::Product => TdrProductType::decode_field(data, decoder).into(),
      TdrObjectKind::Schema => TdrSchemaType::decode_field(data, decoder).into(),
      TdrObjectKind::SchemaProperty => TdrSchemaPropertyType::decode_field(data, decoder).into(),
      TdrObjectKind::DateTime => TdrDateTimeType::decode_field(data, decoder).into(),
      TdrObjectKind::Date => TdrDateType::decode_field(data, decoder).into(),
      TdrObjectKind::Time => TdrTimeType::decode_field(data, decoder).into(),
      TdrObjectKind::Blob => TdrBlobType::decode_field(data, decoder).into(),
      TdrObjectKind::StrObj => TdrStrObj::decode_field(data, decoder).into(),
      TdrObjectKind::BoolObj => TdrBoolObj::decode_field(data, decoder).into(),
      TdrObjectKind::NumObj => TdrNumObj::decode_field(data, decoder).into(),
      TdrObjectKind::MathObj => TdrMathObj::decode_field(data, decoder).into(),
      TdrObjectKind::ListObj => TdrListObj::decode_field(data, decoder).into(),
      TdrObjectKind::DictObj => TdrDictObj::decode_field(data, decoder).into(),
      TdrObjectKind::FuncObj => TdrFuncObj::decode_field(data, decoder).into(),
      TdrObjectKind::ProductObj => TdrProductObj::decode_field(data, decoder).into(),
      TdrObjectKind::DateTimeObj => TdrDateTimeObj::decode_field(data, decoder).into(),
      TdrObjectKind::DateObj => TdrDateObj::decode_field(data, decoder).into(),
      TdrObjectKind::TimeObj => TdrTimeObj::decode_field(data, decoder).into(),
      TdrObjectKind::BlobObj => TdrBlobObj::decode_field(data, decoder).into(),
    }
  }
}
