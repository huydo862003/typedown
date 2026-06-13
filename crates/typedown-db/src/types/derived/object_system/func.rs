use std::collections::HashMap;
use typedown_macros::{query_derived, query_interned};

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TypeMember};
use crate::TypedownDatabase;
use crate::derived::get_builtin_types::get_func_type;

#[query_interned]
pub struct FuncSignature {
  pub params: Vec<Box<dyn TdrTypeLike>>,
  pub ret: Box<dyn TdrTypeLike>,
}

#[query_derived]
pub struct TdrFuncType {
  #[id]
  pub signature: FuncSignature,
}

impl TdrObjectLike for TdrFuncType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_owned_fields(&self, db: &TypedownDatabase) -> HashMap<String, Box<dyn TdrObjectLike>> {
    HashMap::new()
  }
}

impl TdrTypeLike for TdrFuncType {
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
  fn get_owned_fields(&self, db: &TypedownDatabase) -> HashMap<String, Box<dyn TdrObjectLike>> {
    HashMap::new()
  }
}
