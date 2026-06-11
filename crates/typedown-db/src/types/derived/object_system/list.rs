use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike};
use super::func::TdrFuncLike;
use crate::TypedownDatabase;
use crate::derived::get_builtin_types::get_list_type;

pub trait TdrListLike: TdrObjectLike {}

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
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, Box<dyn TdrFuncLike>> {
    HashMap::new()
  }
}

impl TdrListType {
  pub fn get(db: &TypedownDatabase) -> TdrListType {
    get_list_type(db)
  }
}

pub struct TdrListObj<T>(pub Vec<T>);

impl<T: TdrObjectLike> TdrObjectLike for TdrListObj<T> {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrListType::get(db))
  }
  fn get_owned_fields(&self, db: &TypedownDatabase) -> HashMap<String, Box<dyn TdrObjectLike>> {
    HashMap::new()
  }
}

impl<T: TdrObjectLike> TdrListLike for TdrListObj<T> {}
