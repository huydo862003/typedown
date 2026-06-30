use num_enum::TryFromPrimitive;

use super::TdrObjectEnum;
use super::base::TdrObjectLike;
use super::str::TdrStrObj;
use crate::{
  Decodable, Decoder, Encodable, Encoder, QueryDatabase, StableHash, StableHasher, TypedownDatabase,
};

type NativeFn = fn(&TypedownDatabase, TdrObjectEnum, Vec<TdrObjectEnum>) -> Option<TdrObjectEnum>;

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

impl StableHash for NativeFnKind {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    (*self as u8).stable_hash(db, hasher);
  }
}

impl Encodable for NativeFnKind {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_u8(*self as u8);
  }
}

impl Decodable for NativeFnKind {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
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
  this: TdrObjectEnum,
  _args: Vec<TdrObjectEnum>,
) -> Option<TdrObjectEnum> {
  let obj = this.as_tdr_str_obj()?;
  Some(TdrStrObj::new(db, obj.value(db).into()).into())
}

fn num_to_string(
  db: &TypedownDatabase,
  this: TdrObjectEnum,
  _args: Vec<TdrObjectEnum>,
) -> Option<TdrObjectEnum> {
  let obj = this.as_tdr_num_obj()?;
  Some(TdrStrObj::new(db, obj.value(db).to_string()).into())
}

fn bool_to_string(
  db: &TypedownDatabase,
  this: TdrObjectEnum,
  _args: Vec<TdrObjectEnum>,
) -> Option<TdrObjectEnum> {
  let obj = this.as_tdr_bool_obj()?;
  Some(TdrStrObj::new(db, obj.value(db).to_string()).into())
}

fn math_to_string(
  db: &TypedownDatabase,
  this: TdrObjectEnum,
  _args: Vec<TdrObjectEnum>,
) -> Option<TdrObjectEnum> {
  let obj = this.as_tdr_math_obj()?;
  Some(TdrStrObj::new(db, obj.value(db).into()).into())
}

fn object_to_string(
  db: &TypedownDatabase,
  this: TdrObjectEnum,
  _args: Vec<TdrObjectEnum>,
) -> Option<TdrObjectEnum> {
  Some(TdrStrObj::new(db, this.source_path(db).into()).into())
}

fn func_to_string(
  db: &TypedownDatabase,
  this: TdrObjectEnum,
  _args: Vec<TdrObjectEnum>,
) -> Option<TdrObjectEnum> {
  let func = this.as_tdr_func_obj()?;
  Some(TdrStrObj::new(db, func.name(db).into()).into())
}

fn datetime_to_string(
  db: &TypedownDatabase,
  this: TdrObjectEnum,
  _args: Vec<TdrObjectEnum>,
) -> Option<TdrObjectEnum> {
  let obj = this.as_tdr_date_time_obj()?;
  Some(TdrStrObj::new(db, obj.value(db).into()).into())
}

fn date_to_string(
  db: &TypedownDatabase,
  this: TdrObjectEnum,
  _args: Vec<TdrObjectEnum>,
) -> Option<TdrObjectEnum> {
  let obj = this.as_tdr_date_obj()?;
  Some(TdrStrObj::new(db, obj.value(db).into()).into())
}

fn time_to_string(
  db: &TypedownDatabase,
  this: TdrObjectEnum,
  _args: Vec<TdrObjectEnum>,
) -> Option<TdrObjectEnum> {
  let obj = this.as_tdr_time_obj()?;
  Some(TdrStrObj::new(db, obj.value(db).into()).into())
}
