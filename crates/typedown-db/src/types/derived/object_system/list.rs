use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeType, TdrTypeLike};
use super::func::TdrFuncType;
use crate::TypedownDatabase;
use crate::derived::get_builtin_types::get_list_type;
use crate::types::TypeMember;

#[query_derived]
pub struct TdrListType {
  pub elem: Option<Box<dyn TdrTypeLike>>,
}

impl TdrObjectLike for TdrListType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

impl TdrTypeLike for TdrListType {
  fn arity(&self, db: &TypedownDatabase) -> usize {
    if self.elem(db).is_none() { 1 } else { 0 }
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
    assert_eq!(args.len(), self.arity(db), "arity mismatch");
    let mut iter = args.into_iter();
    Box::new(TdrListType::new(db, Some(iter.next().unwrap())))
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
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}
