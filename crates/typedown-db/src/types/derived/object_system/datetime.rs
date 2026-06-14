use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike};
use crate::types::TypeMember;
use super::func::TdrFuncType;
use crate::TypedownDatabase;
use crate::derived::get_builtin_types::{get_date_type, get_datetime_type, get_time_type};

#[query_derived]
pub struct TdrDateTimeType {}

impl TdrObjectLike for TdrDateTimeType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

impl TdrTypeLike for TdrDateTimeType {
  fn get_supertype(&self, db: &TypedownDatabase) -> Option<Box<dyn TdrTypeLike>> {
    Some(Box::new(TdrObjectType::get(db)))
  }
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncType> {
    HashMap::new()
  }
  fn get_owned_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
    todo!()
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
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

pub trait TdrDateLike: TdrObjectLike {}

#[query_derived]
pub struct TdrDateType {}

impl TdrObjectLike for TdrDateType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

impl TdrTypeLike for TdrDateType {
  fn get_supertype(&self, db: &TypedownDatabase) -> Option<Box<dyn TdrTypeLike>> {
    Some(Box::new(TdrObjectType::get(db)))
  }
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncType> {
    HashMap::new()
  }
  fn get_owned_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
    todo!()
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
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

impl TdrDateLike for TdrDateObj {}

pub trait TdrTimeLike: TdrObjectLike {}

#[query_derived]
pub struct TdrTimeType {}

impl TdrObjectLike for TdrTimeType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

impl TdrTypeLike for TdrTimeType {
  fn get_supertype(&self, db: &TypedownDatabase) -> Option<Box<dyn TdrTypeLike>> {
    Some(Box::new(TdrObjectType::get(db)))
  }
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncType> {
    HashMap::new()
  }
  fn get_owned_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
    todo!()
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
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}
