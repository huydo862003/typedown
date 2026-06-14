use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike};
use crate::TypedownDatabase;
use crate::derived::get_builtin_types::get_func_type;
use crate::types::{FuncSignature, TypeMember};

#[query_derived]
pub struct TdrFuncType {
  #[id]
  pub signature: FuncSignature,
}

impl TdrObjectLike for TdrFuncType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

impl TdrTypeLike for TdrFuncType {
  fn arity(&self, db: &TypedownDatabase) -> usize {
    0
  }

  fn get_supertype(&self, db: &TypedownDatabase) -> Option<Box<dyn TdrTypeLike>> {
    Some(Box::new(TdrObjectType::get(db)))
  }
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncType> {
    HashMap::new()
  }
  fn get_owned_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
    todo!()
  }
  fn instantiate(
    &self,
    db: &TypedownDatabase,
    args: Vec<Box<dyn TdrTypeLike>>,
  ) -> Box<dyn TdrTypeLike> {
    Box::new(self.clone())
  }
}

impl TdrFuncType {
  pub fn get(
    db: &TypedownDatabase,
    params: Vec<Box<dyn TdrTypeLike>>,
    ret: Box<dyn TdrTypeLike>,
  ) -> TdrFuncType {
    get_func_type(db, FuncSignature::new(db, params, ret))
  }
}

#[query_derived]
pub struct TdrFuncObj {
  pub name: String,
}

impl TdrObjectLike for TdrFuncObj {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    todo!()
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}
