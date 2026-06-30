use std::any::Any;

use num_enum::TryFromPrimitive;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TdrTypeType};
use super::bool::{TdrBoolObj, TdrBoolType};
use super::datetime::{
  TdrDateObj, TdrDateTimeObj, TdrDateTimeType, TdrDateType, TdrTimeObj, TdrTimeType,
};
use super::dict::{TdrDictObj, TdrDictType};
use super::func::{TdrFuncObj, TdrFuncType};
use super::list::{TdrListObj, TdrListType};
use super::math::{TdrMathObj, TdrMathType};
use super::num::{TdrNumObj, TdrNumType};
use super::product::{TdrProductObj, TdrProductType};
use super::schema::TdrSchemaType;
use super::schema_property::TdrSchemaPropertyType;
use super::str::{TdrStrObj, TdrStrType};
use crate::{Decodable, Decoder, Encodable, Encoder, TypedownDatabase};

#[derive(TryFromPrimitive)]
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

#[derive(TryFromPrimitive)]
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

// Box<dyn TdrTypeLike>
impl Encodable<TypedownDatabase> for Box<dyn TdrTypeLike> {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    let any = self.as_ref() as &dyn Any;
    if let Some(val) = any.downcast_ref::<TdrTypeType>() {
      encoder.emit_u8(TdrTypeKind::Type as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrObjectType>() {
      encoder.emit_u8(TdrTypeKind::Object as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrStrType>() {
      encoder.emit_u8(TdrTypeKind::Str as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrBoolType>() {
      encoder.emit_u8(TdrTypeKind::Bool as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrNumType>() {
      encoder.emit_u8(TdrTypeKind::Num as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrMathType>() {
      encoder.emit_u8(TdrTypeKind::Math as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrListType>() {
      encoder.emit_u8(TdrTypeKind::List as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrDictType>() {
      encoder.emit_u8(TdrTypeKind::Dict as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrFuncType>() {
      encoder.emit_u8(TdrTypeKind::Func as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrProductType>() {
      encoder.emit_u8(TdrTypeKind::Product as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrSchemaType>() {
      encoder.emit_u8(TdrTypeKind::Schema as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrSchemaPropertyType>() {
      encoder.emit_u8(TdrTypeKind::SchemaProperty as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrDateTimeType>() {
      encoder.emit_u8(TdrTypeKind::DateTime as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrDateType>() {
      encoder.emit_u8(TdrTypeKind::Date as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrTimeType>() {
      encoder.emit_u8(TdrTypeKind::Time as u8);
      val.encode(encoder);
    } else {
      panic!("unknown concrete type behind Box<dyn TdrTypeLike>");
    }
  }
}

impl Decodable<TypedownDatabase> for Box<dyn TdrTypeLike> {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    let tag = decoder.read_u8();
    match TdrTypeKind::try_from(tag).unwrap_or_else(|_| panic!("unknown TdrTypeKind tag {tag}")) {
      TdrTypeKind::Type => Box::new(TdrTypeType::decode(decoder)),
      TdrTypeKind::Object => Box::new(TdrObjectType::decode(decoder)),
      TdrTypeKind::Str => Box::new(TdrStrType::decode(decoder)),
      TdrTypeKind::Bool => Box::new(TdrBoolType::decode(decoder)),
      TdrTypeKind::Num => Box::new(TdrNumType::decode(decoder)),
      TdrTypeKind::Math => Box::new(TdrMathType::decode(decoder)),
      TdrTypeKind::List => Box::new(TdrListType::decode(decoder)),
      TdrTypeKind::Dict => Box::new(TdrDictType::decode(decoder)),
      TdrTypeKind::Func => Box::new(TdrFuncType::decode(decoder)),
      TdrTypeKind::Product => Box::new(TdrProductType::decode(decoder)),
      TdrTypeKind::Schema => Box::new(TdrSchemaType::decode(decoder)),
      TdrTypeKind::SchemaProperty => Box::new(TdrSchemaPropertyType::decode(decoder)),
      TdrTypeKind::DateTime => Box::new(TdrDateTimeType::decode(decoder)),
      TdrTypeKind::Date => Box::new(TdrDateType::decode(decoder)),
      TdrTypeKind::Time => Box::new(TdrTimeType::decode(decoder)),
    }
  }
}

// Box<dyn TdrObjectLike>

impl Encodable<TypedownDatabase> for Box<dyn TdrObjectLike> {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    let any = self.as_ref() as &dyn Any;
    // Types (also objects)
    if let Some(val) = any.downcast_ref::<TdrTypeType>() {
      encoder.emit_u8(TdrObjectKind::Type as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrObjectType>() {
      encoder.emit_u8(TdrObjectKind::Object as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrStrType>() {
      encoder.emit_u8(TdrObjectKind::Str as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrBoolType>() {
      encoder.emit_u8(TdrObjectKind::Bool as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrNumType>() {
      encoder.emit_u8(TdrObjectKind::Num as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrMathType>() {
      encoder.emit_u8(TdrObjectKind::Math as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrListType>() {
      encoder.emit_u8(TdrObjectKind::List as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrDictType>() {
      encoder.emit_u8(TdrObjectKind::Dict as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrFuncType>() {
      encoder.emit_u8(TdrObjectKind::Func as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrProductType>() {
      encoder.emit_u8(TdrObjectKind::Product as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrSchemaType>() {
      encoder.emit_u8(TdrObjectKind::Schema as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrSchemaPropertyType>() {
      encoder.emit_u8(TdrObjectKind::SchemaProperty as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrDateTimeType>() {
      encoder.emit_u8(TdrObjectKind::DateTime as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrDateType>() {
      encoder.emit_u8(TdrObjectKind::Date as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrTimeType>() {
      encoder.emit_u8(TdrObjectKind::Time as u8);
      val.encode(encoder);
    // Object-only
    } else if let Some(val) = any.downcast_ref::<TdrStrObj>() {
      encoder.emit_u8(TdrObjectKind::StrObj as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrBoolObj>() {
      encoder.emit_u8(TdrObjectKind::BoolObj as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrNumObj>() {
      encoder.emit_u8(TdrObjectKind::NumObj as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrMathObj>() {
      encoder.emit_u8(TdrObjectKind::MathObj as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrListObj>() {
      encoder.emit_u8(TdrObjectKind::ListObj as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrDictObj>() {
      encoder.emit_u8(TdrObjectKind::DictObj as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrFuncObj>() {
      encoder.emit_u8(TdrObjectKind::FuncObj as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrProductObj>() {
      encoder.emit_u8(TdrObjectKind::ProductObj as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrDateTimeObj>() {
      encoder.emit_u8(TdrObjectKind::DateTimeObj as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrDateObj>() {
      encoder.emit_u8(TdrObjectKind::DateObj as u8);
      val.encode(encoder);
    } else if let Some(val) = any.downcast_ref::<TdrTimeObj>() {
      encoder.emit_u8(TdrObjectKind::TimeObj as u8);
      val.encode(encoder);
    } else {
      panic!("unknown concrete type behind Box<dyn TdrObjectLike>");
    }
  }
}

impl Decodable<TypedownDatabase> for Box<dyn TdrObjectLike> {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    let tag = decoder.read_u8();
    match TdrObjectKind::try_from(tag).unwrap_or_else(|_| panic!("unknown TdrObjectKind tag {tag}"))
    {
      // Types
      TdrObjectKind::Type => Box::new(TdrTypeType::decode(decoder)),
      TdrObjectKind::Object => Box::new(TdrObjectType::decode(decoder)),
      TdrObjectKind::Str => Box::new(TdrStrType::decode(decoder)),
      TdrObjectKind::Bool => Box::new(TdrBoolType::decode(decoder)),
      TdrObjectKind::Num => Box::new(TdrNumType::decode(decoder)),
      TdrObjectKind::Math => Box::new(TdrMathType::decode(decoder)),
      TdrObjectKind::List => Box::new(TdrListType::decode(decoder)),
      TdrObjectKind::Dict => Box::new(TdrDictType::decode(decoder)),
      TdrObjectKind::Func => Box::new(TdrFuncType::decode(decoder)),
      TdrObjectKind::Product => Box::new(TdrProductType::decode(decoder)),
      TdrObjectKind::Schema => Box::new(TdrSchemaType::decode(decoder)),
      TdrObjectKind::SchemaProperty => Box::new(TdrSchemaPropertyType::decode(decoder)),
      TdrObjectKind::DateTime => Box::new(TdrDateTimeType::decode(decoder)),
      TdrObjectKind::Date => Box::new(TdrDateType::decode(decoder)),
      TdrObjectKind::Time => Box::new(TdrTimeType::decode(decoder)),
      // Object-only
      TdrObjectKind::StrObj => Box::new(TdrStrObj::decode(decoder)),
      TdrObjectKind::BoolObj => Box::new(TdrBoolObj::decode(decoder)),
      TdrObjectKind::NumObj => Box::new(TdrNumObj::decode(decoder)),
      TdrObjectKind::MathObj => Box::new(TdrMathObj::decode(decoder)),
      TdrObjectKind::ListObj => Box::new(TdrListObj::decode(decoder)),
      TdrObjectKind::DictObj => Box::new(TdrDictObj::decode(decoder)),
      TdrObjectKind::FuncObj => Box::new(TdrFuncObj::decode(decoder)),
      TdrObjectKind::ProductObj => Box::new(TdrProductObj::decode(decoder)),
      TdrObjectKind::DateTimeObj => Box::new(TdrDateTimeObj::decode(decoder)),
      TdrObjectKind::DateObj => Box::new(TdrDateObj::decode(decoder)),
      TdrObjectKind::TimeObj => Box::new(TdrTimeObj::decode(decoder)),
    }
  }
}
