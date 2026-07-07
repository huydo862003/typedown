mod base;
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

pub use base::*;
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

use typedown_incremental::Id;

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

// Dispatch macro for implementing traits on both enums
macro_rules! dispatch_type_enum {
  ($self:ident, $method:ident($($arg:expr),*)) => {
    match $self {
      TdrTypeEnum::TdrTypeType(v) => v.$method($($arg),*),
      TdrTypeEnum::TdrObjectType(v) => v.$method($($arg),*),
      TdrTypeEnum::TdrBoolType(v) => v.$method($($arg),*),
      TdrTypeEnum::TdrStrType(v) => v.$method($($arg),*),
      TdrTypeEnum::TdrNumType(v) => v.$method($($arg),*),
      TdrTypeEnum::TdrMathType(v) => v.$method($($arg),*),
      TdrTypeEnum::TdrFuncType(v) => v.$method($($arg),*),
      TdrTypeEnum::TdrListType(v) => v.$method($($arg),*),
      TdrTypeEnum::TdrDictType(v) => v.$method($($arg),*),
      TdrTypeEnum::TdrDateTimeType(v) => v.$method($($arg),*),
      TdrTypeEnum::TdrDateType(v) => v.$method($($arg),*),
      TdrTypeEnum::TdrTimeType(v) => v.$method($($arg),*),
      TdrTypeEnum::TdrSchemaType(v) => v.$method($($arg),*),
      TdrTypeEnum::TdrSchemaPropertyType(v) => v.$method($($arg),*),
      TdrTypeEnum::TdrProductType(v) => v.$method($($arg),*),
    }
  };
}

macro_rules! dispatch_object_enum {
  ($self:ident, $method:ident($($arg:expr),*)) => {
    match $self {
      TdrObjectEnum::TdrTypeType(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrObjectType(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrBoolType(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrStrType(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrNumType(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrMathType(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrFuncType(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrListType(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrDictType(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrDateTimeType(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrDateType(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrTimeType(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrSchemaType(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrSchemaPropertyType(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrProductType(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrBoolObj(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrStrObj(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrNumObj(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrMathObj(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrFuncObj(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrListObj(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrDictObj(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrDateTimeObj(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrDateObj(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrTimeObj(v) => v.$method($($arg),*),
      TdrObjectEnum::TdrProductObj(v) => v.$method($($arg),*),
    }
  };
}

impl typedown_incremental::StableHash for TdrTypeEnum {
  fn stable_hash<DB: typedown_incremental::QueryDatabase + ?Sized>(
    &self,
    db: &DB,
    hasher: &mut typedown_incremental::StableHasher,
  ) {
    dispatch_type_enum!(self, stable_hash(db, hasher));
  }
}

impl typedown_incremental::StableHash for TdrObjectEnum {
  fn stable_hash<DB: typedown_incremental::QueryDatabase + ?Sized>(
    &self,
    db: &DB,
    hasher: &mut typedown_incremental::StableHasher,
  ) {
    dispatch_object_enum!(self, stable_hash(db, hasher));
  }
}

use strum::FromRepr;

use typedown_incremental::{Decodable, Decoder, Encodable, Encoder};

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
}

// TdrTypeEnum
impl Encodable for TdrTypeEnum {
  fn encode(&self, buf: &mut Vec<u8>, encoder: &mut Encoder) {
    match self {
      TdrTypeEnum::TdrTypeType(v) => {
        Encoder::emit_u8(buf, TdrTypeKind::Type as u8);
        v.encode(buf, encoder);
      }
      TdrTypeEnum::TdrObjectType(v) => {
        Encoder::emit_u8(buf, TdrTypeKind::Object as u8);
        v.encode(buf, encoder);
      }
      TdrTypeEnum::TdrStrType(v) => {
        Encoder::emit_u8(buf, TdrTypeKind::Str as u8);
        v.encode(buf, encoder);
      }
      TdrTypeEnum::TdrBoolType(v) => {
        Encoder::emit_u8(buf, TdrTypeKind::Bool as u8);
        v.encode(buf, encoder);
      }
      TdrTypeEnum::TdrNumType(v) => {
        Encoder::emit_u8(buf, TdrTypeKind::Num as u8);
        v.encode(buf, encoder);
      }
      TdrTypeEnum::TdrMathType(v) => {
        Encoder::emit_u8(buf, TdrTypeKind::Math as u8);
        v.encode(buf, encoder);
      }
      TdrTypeEnum::TdrListType(v) => {
        Encoder::emit_u8(buf, TdrTypeKind::List as u8);
        v.encode(buf, encoder);
      }
      TdrTypeEnum::TdrDictType(v) => {
        Encoder::emit_u8(buf, TdrTypeKind::Dict as u8);
        v.encode(buf, encoder);
      }
      TdrTypeEnum::TdrFuncType(v) => {
        Encoder::emit_u8(buf, TdrTypeKind::Func as u8);
        v.encode(buf, encoder);
      }
      TdrTypeEnum::TdrProductType(v) => {
        Encoder::emit_u8(buf, TdrTypeKind::Product as u8);
        v.encode(buf, encoder);
      }
      TdrTypeEnum::TdrSchemaType(v) => {
        Encoder::emit_u8(buf, TdrTypeKind::Schema as u8);
        v.encode(buf, encoder);
      }
      TdrTypeEnum::TdrSchemaPropertyType(v) => {
        Encoder::emit_u8(buf, TdrTypeKind::SchemaProperty as u8);
        v.encode(buf, encoder);
      }
      TdrTypeEnum::TdrDateTimeType(v) => {
        Encoder::emit_u8(buf, TdrTypeKind::DateTime as u8);
        v.encode(buf, encoder);
      }
      TdrTypeEnum::TdrDateType(v) => {
        Encoder::emit_u8(buf, TdrTypeKind::Date as u8);
        v.encode(buf, encoder);
      }
      TdrTypeEnum::TdrTimeType(v) => {
        Encoder::emit_u8(buf, TdrTypeKind::Time as u8);
        v.encode(buf, encoder);
      }
    }
  }
}

impl Decodable for TdrTypeEnum {
  fn decode(data: &mut &[u8], decoder: &Decoder) -> Self {
    let tag = Decoder::read_u8(data);
    match TdrTypeKind::from_repr(tag).unwrap_or_else(|| panic!("unknown TdrTypeKind tag {tag}")) {
      TdrTypeKind::Type => TdrTypeType::decode(data, decoder).into(),
      TdrTypeKind::Object => TdrObjectType::decode(data, decoder).into(),
      TdrTypeKind::Str => TdrStrType::decode(data, decoder).into(),
      TdrTypeKind::Bool => TdrBoolType::decode(data, decoder).into(),
      TdrTypeKind::Num => TdrNumType::decode(data, decoder).into(),
      TdrTypeKind::Math => TdrMathType::decode(data, decoder).into(),
      TdrTypeKind::List => TdrListType::decode(data, decoder).into(),
      TdrTypeKind::Dict => TdrDictType::decode(data, decoder).into(),
      TdrTypeKind::Func => TdrFuncType::decode(data, decoder).into(),
      TdrTypeKind::Product => TdrProductType::decode(data, decoder).into(),
      TdrTypeKind::Schema => TdrSchemaType::decode(data, decoder).into(),
      TdrTypeKind::SchemaProperty => TdrSchemaPropertyType::decode(data, decoder).into(),
      TdrTypeKind::DateTime => TdrDateTimeType::decode(data, decoder).into(),
      TdrTypeKind::Date => TdrDateType::decode(data, decoder).into(),
      TdrTypeKind::Time => TdrTimeType::decode(data, decoder).into(),
    }
  }
}

// TdrObjectEnum
impl Encodable for TdrObjectEnum {
  fn encode(&self, buf: &mut Vec<u8>, encoder: &mut Encoder) {
    match self {
      // Types
      TdrObjectEnum::TdrTypeType(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::Type as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrObjectType(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::Object as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrStrType(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::Str as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrBoolType(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::Bool as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrNumType(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::Num as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrMathType(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::Math as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrListType(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::List as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrDictType(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::Dict as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrFuncType(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::Func as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrProductType(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::Product as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrSchemaType(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::Schema as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrSchemaPropertyType(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::SchemaProperty as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrDateTimeType(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::DateTime as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrDateType(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::Date as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrTimeType(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::Time as u8);
        v.encode(buf, encoder);
      }
      // Objects
      TdrObjectEnum::TdrStrObj(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::StrObj as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrBoolObj(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::BoolObj as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrNumObj(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::NumObj as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrMathObj(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::MathObj as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrListObj(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::ListObj as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrDictObj(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::DictObj as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrFuncObj(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::FuncObj as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrProductObj(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::ProductObj as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrDateTimeObj(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::DateTimeObj as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrDateObj(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::DateObj as u8);
        v.encode(buf, encoder);
      }
      TdrObjectEnum::TdrTimeObj(v) => {
        Encoder::emit_u8(buf, TdrObjectKind::TimeObj as u8);
        v.encode(buf, encoder);
      }
    }
  }
}

impl Decodable for TdrObjectEnum {
  fn decode(data: &mut &[u8], decoder: &Decoder) -> Self {
    let tag = Decoder::read_u8(data);
    match TdrObjectKind::from_repr(tag).unwrap_or_else(|| panic!("unknown TdrObjectKind tag {tag}"))
    {
      // Types
      TdrObjectKind::Type => TdrTypeType::decode(data, decoder).into(),
      TdrObjectKind::Object => TdrObjectType::decode(data, decoder).into(),
      TdrObjectKind::Str => TdrStrType::decode(data, decoder).into(),
      TdrObjectKind::Bool => TdrBoolType::decode(data, decoder).into(),
      TdrObjectKind::Num => TdrNumType::decode(data, decoder).into(),
      TdrObjectKind::Math => TdrMathType::decode(data, decoder).into(),
      TdrObjectKind::List => TdrListType::decode(data, decoder).into(),
      TdrObjectKind::Dict => TdrDictType::decode(data, decoder).into(),
      TdrObjectKind::Func => TdrFuncType::decode(data, decoder).into(),
      TdrObjectKind::Product => TdrProductType::decode(data, decoder).into(),
      TdrObjectKind::Schema => TdrSchemaType::decode(data, decoder).into(),
      TdrObjectKind::SchemaProperty => TdrSchemaPropertyType::decode(data, decoder).into(),
      TdrObjectKind::DateTime => TdrDateTimeType::decode(data, decoder).into(),
      TdrObjectKind::Date => TdrDateType::decode(data, decoder).into(),
      TdrObjectKind::Time => TdrTimeType::decode(data, decoder).into(),
      // Objects
      TdrObjectKind::StrObj => TdrStrObj::decode(data, decoder).into(),
      TdrObjectKind::BoolObj => TdrBoolObj::decode(data, decoder).into(),
      TdrObjectKind::NumObj => TdrNumObj::decode(data, decoder).into(),
      TdrObjectKind::MathObj => TdrMathObj::decode(data, decoder).into(),
      TdrObjectKind::ListObj => TdrListObj::decode(data, decoder).into(),
      TdrObjectKind::DictObj => TdrDictObj::decode(data, decoder).into(),
      TdrObjectKind::FuncObj => TdrFuncObj::decode(data, decoder).into(),
      TdrObjectKind::ProductObj => TdrProductObj::decode(data, decoder).into(),
      TdrObjectKind::DateTimeObj => TdrDateTimeObj::decode(data, decoder).into(),
      TdrObjectKind::DateObj => TdrDateObj::decode(data, decoder).into(),
      TdrObjectKind::TimeObj => TdrTimeObj::decode(data, decoder).into(),
    }
  }
}
