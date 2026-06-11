use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike};
use super::func::TdrFuncLike;
use crate::TypedownDatabase;
use crate::derived::get_builtin_types::get_enum_type;

pub trait TdrEnumLike: TdrObjectLike {}

#[query_derived]
pub struct TdrEnumType {}

impl TdrObjectLike for TdrEnumType {
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

impl TdrTypeLike for TdrEnumType {
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

impl TdrEnumType {
  pub fn get(db: &TypedownDatabase) -> TdrEnumType {
    get_enum_type(db)
  }
}

pub struct TdrEnumObj {
  pub variants: Vec<String>,
  pub value: String,
}

impl TdrObjectLike for TdrEnumObj {
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

impl TdrEnumLike for TdrEnumObj {}
