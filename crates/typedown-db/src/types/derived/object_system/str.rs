use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike};
use crate::types::TypeMember;
use super::func::TdrFuncType;
use crate::TypedownDatabase;
use crate::derived::get_builtin_types::get_str_type;

#[query_derived]
pub struct TdrStrType {}

impl TdrObjectLike for TdrStrType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

impl TdrTypeLike for TdrStrType {
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

impl TdrStrType {
  pub fn get(db: &TypedownDatabase) -> TdrStrType {
    get_str_type(db)
  }
}

#[query_derived]
pub struct TdrStrObj {
  pub value: String,
}

impl TdrObjectLike for TdrStrObj {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrStrType::get(db))
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}
