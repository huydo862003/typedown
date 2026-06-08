use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike};
use super::func::TdrFuncLike;
use crate::TypedownDatabase;

pub trait TdrDateTimeLike: TdrObjectLike {}

#[query_derived]
pub struct TdrDateTimeType {}

impl TdrObjectLike for TdrDateTimeType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_owned_fields(
    &self,
    db: &TypedownDatabase,
  ) -> HashMap<String, Box<dyn TdrObjectLike>> {
    HashMap::new()
  }
}

impl TdrTypeLike for TdrDateTimeType {
  fn get_supertype(&self, db: &TypedownDatabase) -> Option<Box<dyn TdrTypeLike>> {
    Some(Box::new(TdrObjectType::get(db)))
  }
  fn get_vtable(
    &self,
    db: &TypedownDatabase,
  ) -> HashMap<String, Box<dyn TdrFuncLike>> {
    HashMap::new()
  }
}

impl TdrDateTimeType {
  pub fn get(db: &TypedownDatabase) -> TdrDateTimeType {
    todo!()
  }
}

pub struct TdrDateTimeObj(pub String);

impl TdrObjectLike for TdrDateTimeObj {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    todo!()
  }
  fn get_owned_fields(
    &self,
    db: &TypedownDatabase,
  ) -> HashMap<String, Box<dyn TdrObjectLike>> {
    HashMap::new()
  }
}

impl TdrDateTimeLike for TdrDateTimeObj {}

pub trait TdrDateLike: TdrObjectLike {}

#[query_derived]
pub struct TdrDateType {}

impl TdrObjectLike for TdrDateType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_owned_fields(
    &self,
    db: &TypedownDatabase,
  ) -> HashMap<String, Box<dyn TdrObjectLike>> {
    HashMap::new()
  }
}

impl TdrTypeLike for TdrDateType {
  fn get_supertype(&self, db: &TypedownDatabase) -> Option<Box<dyn TdrTypeLike>> {
    Some(Box::new(TdrObjectType::get(db)))
  }
  fn get_vtable(
    &self,
    db: &TypedownDatabase,
  ) -> HashMap<String, Box<dyn TdrFuncLike>> {
    HashMap::new()
  }
}

impl TdrDateType {
  pub fn get(db: &TypedownDatabase) -> TdrDateType {
    todo!()
  }
}

pub struct TdrDateObj(pub String);

impl TdrObjectLike for TdrDateObj {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    todo!()
  }
  fn get_owned_fields(
    &self,
    db: &TypedownDatabase,
  ) -> HashMap<String, Box<dyn TdrObjectLike>> {
    HashMap::new()
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
  fn get_owned_fields(
    &self,
    db: &TypedownDatabase,
  ) -> HashMap<String, Box<dyn TdrObjectLike>> {
    HashMap::new()
  }
}

impl TdrTypeLike for TdrTimeType {
  fn get_supertype(&self, db: &TypedownDatabase) -> Option<Box<dyn TdrTypeLike>> {
    Some(Box::new(TdrObjectType::get(db)))
  }
  fn get_vtable(
    &self,
    db: &TypedownDatabase,
  ) -> HashMap<String, Box<dyn TdrFuncLike>> {
    HashMap::new()
  }
}

impl TdrTimeType {
  pub fn get(db: &TypedownDatabase) -> TdrTimeType {
    todo!()
  }
}

pub struct TdrTimeObj(pub String);

impl TdrObjectLike for TdrTimeObj {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    todo!()
  }
  fn get_owned_fields(
    &self,
    db: &TypedownDatabase,
  ) -> HashMap<String, Box<dyn TdrObjectLike>> {
    HashMap::new()
  }
}

impl TdrTimeLike for TdrTimeObj {}
