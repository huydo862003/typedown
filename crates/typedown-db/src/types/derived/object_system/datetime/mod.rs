mod utils;

use std::any::Any;
use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrTypeLike, TdrTypeType};
use super::func::TdrFuncObj;
use super::native_fn::NativeFnKind;
use super::str::{TdrStrObj, TdrStrType};
use crate::derived::get_builtin_types::{get_date_type, get_datetime_type, get_time_type};
use crate::types::{FuncSignature, InstResult, TypeMember};
use crate::{
  Decodable, Decoder, Encodable, Encoder, Id, StableHash, StableHasher, TypedownDatabase,
};
pub(crate) use utils::{is_valid_iso_date, is_valid_iso_datetime, is_valid_iso_time};

#[query_derived]
pub struct TdrDateTimeType {}

impl TdrObjectLike for TdrDateTimeType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
  fn source_path(&self, _db: &TypedownDatabase) -> String {
    "@builtin::datetime".to_string()
  }

  fn as_type(&self) -> Option<Box<dyn TdrTypeLike>> {
    Some(Box::new(self.clone()))
  }
}

impl TdrTypeLike for TdrDateTimeType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }

  fn get_supertype(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrStrType::get(db))
  }
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncObj> {
    let sig = FuncSignature::new(db, vec![], Box::new(TdrStrType::get(db)));
    let func_obj = TdrFuncObj::new(
      db,
      "to_string".to_string(),
      Box::new(TdrDateTimeType::get(db)),
      sig,
      NativeFnKind::DateTimeToString,
    );
    HashMap::from([("to_string".to_string(), func_obj)])
  }
  fn get_owned_field_type(&self, _db: &TypedownDatabase, _name: &str) -> Option<TypeMember> {
    None
  }
  fn instantiate(&self, db: &TypedownDatabase, args: Vec<Box<dyn TdrTypeLike>>) -> InstResult {
    assert_eq!(args.len(), self.arity(db), "arity mismatch");
    InstResult::new(db, Box::new(self.clone()), vec![])
  }

  fn get_type_args(&self, _db: &TypedownDatabase) -> Vec<Box<dyn TdrTypeLike>> {
    vec![]
  }

  fn is_compatible_with(&self, _db: &TypedownDatabase, actual: &dyn TdrTypeLike) -> bool {
    self.as_id() == actual.as_id()
  }

  fn construct(
    &self,
    db: &TypedownDatabase,
    args: Vec<Box<dyn TdrObjectLike>>,
  ) -> Option<Box<dyn TdrObjectLike>> {
    let arg = args.into_iter().next()?;
    let str_obj = (arg.as_ref() as &dyn Any).downcast_ref::<TdrStrObj>()?;
    let val = str_obj.value(db);
    if !is_valid_iso_datetime(&val) {
      return None;
    }
    Some(Box::new(TdrDateTimeObj::new(db, val)))
  }

  fn display_name(&self, _db: &TypedownDatabase) -> String {
    "datetime".to_string()
  }
}

impl TdrDateTimeType {
  pub fn get(db: &TypedownDatabase) -> TdrDateTimeType {
    get_datetime_type(db)
  }
}

#[query_derived]
pub struct TdrDateTimeObj {
  pub value: String,
}

impl TdrObjectLike for TdrDateTimeObj {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrDateTimeType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }

  fn eq(&self, db: &TypedownDatabase, other: &dyn TdrObjectLike) -> bool {
    match (other as &dyn std::any::Any).downcast_ref::<TdrDateTimeObj>() {
      Some(other) => self.value(db) == other.value(db),
      None => self.as_id() == other.as_id(),
    }
  }
  fn lt(&self, db: &TypedownDatabase, other: &dyn TdrObjectLike) -> bool {
    match (other as &dyn std::any::Any).downcast_ref::<TdrDateTimeObj>() {
      Some(other) => self.value(db) < other.value(db),
      None => self.as_id() < other.as_id(),
    }
  }
  fn gt(&self, db: &TypedownDatabase, other: &dyn TdrObjectLike) -> bool {
    match (other as &dyn std::any::Any).downcast_ref::<TdrDateTimeObj>() {
      Some(other) => self.value(db) > other.value(db),
      None => self.as_id() > other.as_id(),
    }
  }
  fn le(&self, db: &TypedownDatabase, other: &dyn TdrObjectLike) -> bool {
    match (other as &dyn std::any::Any).downcast_ref::<TdrDateTimeObj>() {
      Some(other) => self.value(db) <= other.value(db),
      None => self.as_id() <= other.as_id(),
    }
  }
  fn ge(&self, db: &TypedownDatabase, other: &dyn TdrObjectLike) -> bool {
    match (other as &dyn std::any::Any).downcast_ref::<TdrDateTimeObj>() {
      Some(other) => self.value(db) >= other.value(db),
      None => self.as_id() >= other.as_id(),
    }
  }
  fn source_path(&self, db: &TypedownDatabase) -> String {
    self.get_type(db).source_path(db)
  }
}

pub trait TdrDateLike: TdrObjectLike {}

#[query_derived]
pub struct TdrDateType {}

impl TdrObjectLike for TdrDateType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
  fn source_path(&self, _db: &TypedownDatabase) -> String {
    "@builtin::date".to_string()
  }

  fn as_type(&self) -> Option<Box<dyn TdrTypeLike>> {
    Some(Box::new(self.clone()))
  }
}

impl TdrTypeLike for TdrDateType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }

  fn get_supertype(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrStrType::get(db))
  }
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncObj> {
    let sig = FuncSignature::new(db, vec![], Box::new(TdrStrType::get(db)));
    let func_obj = TdrFuncObj::new(
      db,
      "to_string".to_string(),
      Box::new(TdrDateType::get(db)),
      sig,
      NativeFnKind::DateToString,
    );
    HashMap::from([("to_string".to_string(), func_obj)])
  }
  fn get_owned_field_type(&self, _db: &TypedownDatabase, _name: &str) -> Option<TypeMember> {
    None
  }
  fn instantiate(&self, db: &TypedownDatabase, args: Vec<Box<dyn TdrTypeLike>>) -> InstResult {
    assert_eq!(args.len(), self.arity(db), "arity mismatch");
    InstResult::new(db, Box::new(self.clone()), vec![])
  }

  fn get_type_args(&self, _db: &TypedownDatabase) -> Vec<Box<dyn TdrTypeLike>> {
    vec![]
  }

  fn is_compatible_with(&self, _db: &TypedownDatabase, actual: &dyn TdrTypeLike) -> bool {
    self.as_id() == actual.as_id()
  }

  fn construct(
    &self,
    db: &TypedownDatabase,
    args: Vec<Box<dyn TdrObjectLike>>,
  ) -> Option<Box<dyn TdrObjectLike>> {
    let arg = args.into_iter().next()?;
    let str_obj = (arg.as_ref() as &dyn Any).downcast_ref::<TdrStrObj>()?;
    let val = str_obj.value(db);
    if !is_valid_iso_date(&val) {
      return None;
    }
    Some(Box::new(TdrDateObj::new(db, val)))
  }

  fn display_name(&self, _db: &TypedownDatabase) -> String {
    "date".to_string()
  }
}

impl TdrDateType {
  pub fn get(db: &TypedownDatabase) -> TdrDateType {
    get_date_type(db)
  }
}

#[query_derived]
pub struct TdrDateObj {
  pub value: String,
}

impl TdrObjectLike for TdrDateObj {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrDateType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }

  fn eq(&self, db: &TypedownDatabase, other: &dyn TdrObjectLike) -> bool {
    match (other as &dyn std::any::Any).downcast_ref::<TdrDateObj>() {
      Some(other) => self.value(db) == other.value(db),
      None => self.as_id() == other.as_id(),
    }
  }
  fn lt(&self, db: &TypedownDatabase, other: &dyn TdrObjectLike) -> bool {
    match (other as &dyn std::any::Any).downcast_ref::<TdrDateObj>() {
      Some(other) => self.value(db) < other.value(db),
      None => self.as_id() < other.as_id(),
    }
  }
  fn gt(&self, db: &TypedownDatabase, other: &dyn TdrObjectLike) -> bool {
    match (other as &dyn std::any::Any).downcast_ref::<TdrDateObj>() {
      Some(other) => self.value(db) > other.value(db),
      None => self.as_id() > other.as_id(),
    }
  }
  fn le(&self, db: &TypedownDatabase, other: &dyn TdrObjectLike) -> bool {
    match (other as &dyn std::any::Any).downcast_ref::<TdrDateObj>() {
      Some(other) => self.value(db) <= other.value(db),
      None => self.as_id() <= other.as_id(),
    }
  }
  fn ge(&self, db: &TypedownDatabase, other: &dyn TdrObjectLike) -> bool {
    match (other as &dyn std::any::Any).downcast_ref::<TdrDateObj>() {
      Some(other) => self.value(db) >= other.value(db),
      None => self.as_id() >= other.as_id(),
    }
  }
  fn source_path(&self, db: &TypedownDatabase) -> String {
    self.get_type(db).source_path(db)
  }
}

impl TdrDateLike for TdrDateObj {}

pub trait TdrTimeLike: TdrObjectLike {}

#[query_derived]
pub struct TdrTimeType {}

impl TdrObjectLike for TdrTimeType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
  fn source_path(&self, _db: &TypedownDatabase) -> String {
    "@builtin::time".to_string()
  }

  fn as_type(&self) -> Option<Box<dyn TdrTypeLike>> {
    Some(Box::new(self.clone()))
  }
}

impl TdrTypeLike for TdrTimeType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }

  fn get_supertype(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrStrType::get(db))
  }
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncObj> {
    let sig = FuncSignature::new(db, vec![], Box::new(TdrStrType::get(db)));
    let func_obj = TdrFuncObj::new(
      db,
      "to_string".to_string(),
      Box::new(TdrTimeType::get(db)),
      sig,
      NativeFnKind::TimeToString,
    );
    HashMap::from([("to_string".to_string(), func_obj)])
  }
  fn get_owned_field_type(&self, _db: &TypedownDatabase, _name: &str) -> Option<TypeMember> {
    None
  }
  fn instantiate(&self, db: &TypedownDatabase, args: Vec<Box<dyn TdrTypeLike>>) -> InstResult {
    assert_eq!(args.len(), self.arity(db), "arity mismatch");
    InstResult::new(db, Box::new(self.clone()), vec![])
  }

  fn get_type_args(&self, _db: &TypedownDatabase) -> Vec<Box<dyn TdrTypeLike>> {
    vec![]
  }

  fn is_compatible_with(&self, _db: &TypedownDatabase, actual: &dyn TdrTypeLike) -> bool {
    self.as_id() == actual.as_id()
  }

  fn construct(
    &self,
    db: &TypedownDatabase,
    args: Vec<Box<dyn TdrObjectLike>>,
  ) -> Option<Box<dyn TdrObjectLike>> {
    let arg = args.into_iter().next()?;
    let str_obj = (arg.as_ref() as &dyn Any).downcast_ref::<TdrStrObj>()?;
    let val = str_obj.value(db);
    if !is_valid_iso_time(&val) {
      return None;
    }
    Some(Box::new(TdrTimeObj::new(db, val)))
  }

  fn display_name(&self, _db: &TypedownDatabase) -> String {
    "time".to_string()
  }
}

impl TdrTimeType {
  pub fn get(db: &TypedownDatabase) -> TdrTimeType {
    get_time_type(db)
  }
}

#[query_derived]
pub struct TdrTimeObj {
  pub value: String,
}

impl TdrObjectLike for TdrTimeObj {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTimeType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }

  fn source_path(&self, db: &TypedownDatabase) -> String {
    self.get_type(db).source_path(db)
  }

  fn eq(&self, db: &TypedownDatabase, other: &dyn TdrObjectLike) -> bool {
    match (other as &dyn std::any::Any).downcast_ref::<TdrTimeObj>() {
      Some(other) => self.value(db) == other.value(db),
      None => self.as_id() == other.as_id(),
    }
  }
  fn lt(&self, db: &TypedownDatabase, other: &dyn TdrObjectLike) -> bool {
    match (other as &dyn std::any::Any).downcast_ref::<TdrTimeObj>() {
      Some(other) => self.value(db) < other.value(db),
      None => self.as_id() < other.as_id(),
    }
  }
  fn gt(&self, db: &TypedownDatabase, other: &dyn TdrObjectLike) -> bool {
    match (other as &dyn std::any::Any).downcast_ref::<TdrTimeObj>() {
      Some(other) => self.value(db) > other.value(db),
      None => self.as_id() > other.as_id(),
    }
  }
  fn le(&self, db: &TypedownDatabase, other: &dyn TdrObjectLike) -> bool {
    match (other as &dyn std::any::Any).downcast_ref::<TdrTimeObj>() {
      Some(other) => self.value(db) <= other.value(db),
      None => self.as_id() <= other.as_id(),
    }
  }
  fn ge(&self, db: &TypedownDatabase, other: &dyn TdrObjectLike) -> bool {
    match (other as &dyn std::any::Any).downcast_ref::<TdrTimeObj>() {
      Some(other) => self.value(db) >= other.value(db),
      None => self.as_id() >= other.as_id(),
    }
  }
}

impl StableHash<TypedownDatabase> for TdrDateTimeType {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.source_path(db).stable_hash(db, hasher);
  }
}

impl StableHash<TypedownDatabase> for TdrDateTimeObj {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.value(db).stable_hash(db, hasher);
  }
}

impl StableHash<TypedownDatabase> for TdrDateType {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.source_path(db).stable_hash(db, hasher);
  }
}

impl StableHash<TypedownDatabase> for TdrDateObj {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.value(db).stable_hash(db, hasher);
  }
}

impl StableHash<TypedownDatabase> for TdrTimeType {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.source_path(db).stable_hash(db, hasher);
  }
}

impl StableHash<TypedownDatabase> for TdrTimeObj {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.value(db).stable_hash(db, hasher);
  }
}

impl Encodable<TypedownDatabase> for TdrDateTimeType {
  fn encode(&self, _encoder: &mut Encoder<TypedownDatabase>) {}
}

impl Decodable<TypedownDatabase> for TdrDateTimeType {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    TdrDateTimeType::get(decoder.db)
  }
}

impl Encodable<TypedownDatabase> for TdrDateType {
  fn encode(&self, _encoder: &mut Encoder<TypedownDatabase>) {}
}

impl Decodable<TypedownDatabase> for TdrDateType {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    TdrDateType::get(decoder.db)
  }
}

impl Encodable<TypedownDatabase> for TdrTimeType {
  fn encode(&self, _encoder: &mut Encoder<TypedownDatabase>) {}
}

impl Decodable<TypedownDatabase> for TdrTimeType {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    TdrTimeType::get(decoder.db)
  }
}

impl Encodable<TypedownDatabase> for TdrDateTimeObj {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    self.value(encoder.db).encode(encoder);
  }
}

impl Decodable<TypedownDatabase> for TdrDateTimeObj {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    TdrDateTimeObj::new(decoder.db, String::decode(decoder))
  }
}

impl Encodable<TypedownDatabase> for TdrDateObj {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    self.value(encoder.db).encode(encoder);
  }
}

impl Decodable<TypedownDatabase> for TdrDateObj {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    TdrDateObj::new(decoder.db, String::decode(decoder))
  }
}

impl Encodable<TypedownDatabase> for TdrTimeObj {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    self.value(encoder.db).encode(encoder);
  }
}

impl Decodable<TypedownDatabase> for TdrTimeObj {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    TdrTimeObj::new(decoder.db, String::decode(decoder))
  }
}
