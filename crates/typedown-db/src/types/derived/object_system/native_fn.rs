use std::any::Any;

use num_enum::TryFromPrimitive;

use super::base::TdrObjectLike;
use super::bool::TdrBoolObj;
use super::datetime::{TdrDateObj, TdrDateTimeObj, TdrTimeObj};
use super::func::TdrFuncObj;
use super::math::TdrMathObj;
use super::num::TdrNumObj;
use super::str::TdrStrObj;
use crate::{Decodable, Decoder, Encodable, Encoder, StableHash, StableHasher, TypedownDatabase};

type NativeFn = fn(
  &TypedownDatabase,
  Box<dyn TdrObjectLike>,
  Vec<Box<dyn TdrObjectLike>>,
) -> Option<Box<dyn TdrObjectLike>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum NativeFnKind {
  StrToString = 0,
  NumToString = 1,
  BoolToString = 2,
  MathToString = 3,
  ObjectToString = 4,
  FuncToString = 5,
  DateTimeToString = 6,
  DateToString = 7,
  TimeToString = 8,
}

impl StableHash<TypedownDatabase> for NativeFnKind {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    (*self as u8).stable_hash(db, hasher);
  }
}

impl Encodable<TypedownDatabase> for NativeFnKind {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    encoder.emit_u8(*self as u8);
  }
}

impl Decodable<TypedownDatabase> for NativeFnKind {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    let tag = decoder.read_u8();
    NativeFnKind::try_from(tag).unwrap_or_else(|_| panic!("unknown NativeFnKind tag {tag}"))
  }
}

impl NativeFnKind {
  pub fn resolve(self) -> NativeFn {
    match self {
      NativeFnKind::StrToString => str_to_string,
      NativeFnKind::NumToString => num_to_string,
      NativeFnKind::BoolToString => bool_to_string,
      NativeFnKind::MathToString => math_to_string,
      NativeFnKind::ObjectToString => object_to_string,
      NativeFnKind::FuncToString => func_to_string,
      NativeFnKind::DateTimeToString => datetime_to_string,
      NativeFnKind::DateToString => date_to_string,
      NativeFnKind::TimeToString => time_to_string,
    }
  }
}

fn str_to_string(
  db: &TypedownDatabase,
  this: Box<dyn TdrObjectLike>,
  _args: Vec<Box<dyn TdrObjectLike>>,
) -> Option<Box<dyn TdrObjectLike>> {
  let obj = (this.as_ref() as &dyn Any).downcast_ref::<TdrStrObj>()?;
  Some(Box::new(TdrStrObj::new(db, obj.value(db))))
}

fn num_to_string(
  db: &TypedownDatabase,
  this: Box<dyn TdrObjectLike>,
  _args: Vec<Box<dyn TdrObjectLike>>,
) -> Option<Box<dyn TdrObjectLike>> {
  let obj = (this.as_ref() as &dyn Any).downcast_ref::<TdrNumObj>()?;
  Some(Box::new(TdrStrObj::new(db, obj.value(db).to_string())))
}

fn bool_to_string(
  db: &TypedownDatabase,
  this: Box<dyn TdrObjectLike>,
  _args: Vec<Box<dyn TdrObjectLike>>,
) -> Option<Box<dyn TdrObjectLike>> {
  let obj = (this.as_ref() as &dyn Any).downcast_ref::<TdrBoolObj>()?;
  Some(Box::new(TdrStrObj::new(db, obj.value(db).to_string())))
}

fn math_to_string(
  db: &TypedownDatabase,
  this: Box<dyn TdrObjectLike>,
  _args: Vec<Box<dyn TdrObjectLike>>,
) -> Option<Box<dyn TdrObjectLike>> {
  let obj = (this.as_ref() as &dyn Any).downcast_ref::<TdrMathObj>()?;
  Some(Box::new(TdrStrObj::new(db, obj.value(db))))
}

fn object_to_string(
  db: &TypedownDatabase,
  this: Box<dyn TdrObjectLike>,
  _args: Vec<Box<dyn TdrObjectLike>>,
) -> Option<Box<dyn TdrObjectLike>> {
  Some(Box::new(TdrStrObj::new(db, this.source_path(db))))
}

fn func_to_string(
  db: &TypedownDatabase,
  this: Box<dyn TdrObjectLike>,
  _args: Vec<Box<dyn TdrObjectLike>>,
) -> Option<Box<dyn TdrObjectLike>> {
  let func = (this.as_ref() as &dyn Any).downcast_ref::<TdrFuncObj>()?;
  Some(Box::new(TdrStrObj::new(db, func.name(db))))
}

fn datetime_to_string(
  db: &TypedownDatabase,
  this: Box<dyn TdrObjectLike>,
  _args: Vec<Box<dyn TdrObjectLike>>,
) -> Option<Box<dyn TdrObjectLike>> {
  let obj = (this.as_ref() as &dyn Any).downcast_ref::<TdrDateTimeObj>()?;
  Some(Box::new(TdrStrObj::new(db, obj.value(db))))
}

fn date_to_string(
  db: &TypedownDatabase,
  this: Box<dyn TdrObjectLike>,
  _args: Vec<Box<dyn TdrObjectLike>>,
) -> Option<Box<dyn TdrObjectLike>> {
  let obj = (this.as_ref() as &dyn Any).downcast_ref::<TdrDateObj>()?;
  Some(Box::new(TdrStrObj::new(db, obj.value(db))))
}

fn time_to_string(
  db: &TypedownDatabase,
  this: Box<dyn TdrObjectLike>,
  _args: Vec<Box<dyn TdrObjectLike>>,
) -> Option<Box<dyn TdrObjectLike>> {
  let obj = (this.as_ref() as &dyn Any).downcast_ref::<TdrTimeObj>()?;
  Some(Box::new(TdrStrObj::new(db, obj.value(db))))
}
