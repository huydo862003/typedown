use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TypeMember};
use super::func::TdrFuncType;
use crate::TypedownDatabase;
pub trait TdrEnumLike: TdrObjectLike {}

#[query_derived]
pub struct TdrEnumType {}

impl TdrObjectLike for TdrEnumType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

impl TdrTypeLike for TdrEnumType {
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
