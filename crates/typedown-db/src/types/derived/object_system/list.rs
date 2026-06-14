use crate::Id;
use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TypeMember};
use super::func::TdrFuncType;
use crate::TypedownDatabase;
use crate::derived::get_builtin_types::get_list_type;

#[query_derived]
pub struct TdrListType {}

impl TdrObjectLike for TdrListType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_owned_fields(&self, db: &TypedownDatabase) -> HashMap<String, Box<dyn TdrObjectLike>> {
    HashMap::new()
  }
}

impl TdrTypeLike for TdrListType {
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

impl TdrListType {
  pub fn get(db: &TypedownDatabase) -> TdrListType {
    get_list_type(db)
  }
}

#[query_derived]
pub struct TdrListObj {
  pub items: Vec<Box<dyn TdrObjectLike>>,
}

impl TdrObjectLike for TdrListObj {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrListType::get(db))
  }
  fn get_owned_fields(&self, db: &TypedownDatabase) -> HashMap<String, Box<dyn TdrObjectLike>> {
    HashMap::new()
  }
}
