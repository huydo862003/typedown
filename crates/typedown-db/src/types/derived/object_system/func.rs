use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike};
use crate::TypedownDatabase;

pub trait TdrFuncLike: TdrObjectLike {}

#[query_derived]
pub struct TdrFuncType {}

impl TdrObjectLike for TdrFuncType {
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

impl TdrTypeLike for TdrFuncType {
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

impl TdrFuncType {
  pub fn get(db: &TypedownDatabase) -> TdrFuncType {
    todo!()
  }
}

pub struct TdrFuncObj {
  pub name: String,
}

impl TdrObjectLike for TdrFuncObj {
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

impl TdrFuncLike for TdrFuncObj {}
