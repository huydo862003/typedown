use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike};
use super::func::TdrFuncLike;
use crate::TypedownDatabase;
use crate::derived::get_builtin_types::{get_bool_type, get_false, get_true};

pub trait TdrBoolLike: TdrObjectLike {}

#[query_derived]
pub struct TdrBoolType {}

impl TdrObjectLike for TdrBoolType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_owned_fields(&self, db: &TypedownDatabase) -> HashMap<String, Box<dyn TdrObjectLike>> {
    HashMap::new()
  }
}

impl TdrTypeLike for TdrBoolType {
  fn get_supertype(&self, db: &TypedownDatabase) -> Option<Box<dyn TdrTypeLike>> {
    Some(Box::new(TdrObjectType::get(db)))
  }
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, Box<dyn TdrFuncLike>> {
    HashMap::new()
  }
}

impl TdrBoolType {
  pub fn get(db: &TypedownDatabase) -> TdrBoolType {
    get_bool_type(db)
  }
}

#[query_derived]
pub struct TdrBoolObj {
  pub value: bool,
}

impl TdrObjectLike for TdrBoolObj {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrBoolType::get(db))
  }
  fn get_owned_fields(&self, db: &TypedownDatabase) -> HashMap<String, Box<dyn TdrObjectLike>> {
    HashMap::new()
  }
}

impl TdrBoolObj {
  pub fn get_true(db: &TypedownDatabase) -> TdrBoolObj {
    get_true(db)
  }

  pub fn get_false(db: &TypedownDatabase) -> TdrBoolObj {
    get_false(db)
  }
}

impl TdrBoolLike for TdrBoolObj {}
