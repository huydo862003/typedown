use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeType, TdrTypeLike};
use super::func::TdrFuncType;
use crate::TypedownDatabase;
use crate::derived::get_builtin_types::{get_bool_type, get_false, get_true};
use crate::types::TypeMember;

#[query_derived]
pub struct TdrBoolType {}

impl TdrObjectLike for TdrBoolType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

impl TdrTypeLike for TdrBoolType {
  fn arity(&self, db: &TypedownDatabase) -> usize {
    0
  }

  fn get_supertype(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncType> {
    HashMap::new()
  }
  fn get_owned_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
    None
  }
  fn instantiate(
    &self,
    db: &TypedownDatabase,
    args: Vec<Box<dyn TdrTypeLike>>,
  ) -> Box<dyn TdrTypeLike> {
    Box::new(self.clone())
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
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
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
