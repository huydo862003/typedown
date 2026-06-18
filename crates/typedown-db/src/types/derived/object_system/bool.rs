use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TdrTypeType};
use super::func::TdrFuncType;
use crate::derived::get_builtin_types::{get_bool_type, get_false, get_true};
use crate::types::{HirValue, HirValueKind, TypeMember};
use crate::{Id, TypedownDatabase};

#[query_derived]
pub struct TdrBoolType {}

impl TdrObjectLike for TdrBoolType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

impl TdrTypeLike for TdrBoolType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }

  fn get_supertype(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_vtable(&self, _db: &TypedownDatabase) -> HashMap<String, TdrFuncType> {
    HashMap::new()
  }
  fn get_owned_field_type(&self, _db: &TypedownDatabase, _name: &str) -> Option<TypeMember> {
    None
  }
  fn instantiate(
    &self,
    db: &TypedownDatabase,
    args: Vec<Box<dyn TdrTypeLike>>,
  ) -> Box<dyn TdrTypeLike> {
    assert_eq!(args.len(), self.arity(db), "arity mismatch");
    Box::new(self.clone())
  }

  fn get_type_args(&self, _db: &TypedownDatabase) -> Vec<Box<dyn TdrTypeLike>> {
    vec![]
  }

  fn is_compatible_with(&self, _db: &TypedownDatabase, actual: &dyn TdrTypeLike) -> bool {
    self.as_id() == actual.as_id()
  }

  fn construct(&self, db: &TypedownDatabase, hir: HirValue) -> Option<Box<dyn TdrObjectLike>> {
    match hir.kind(db) {
      HirValueKind::Bool(val) => Some(Box::new(TdrBoolObj::new(db, val))),
      _ => None,
    }
  }

  fn display_name(&self, _db: &TypedownDatabase) -> String {
    "boolean".to_string()
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
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
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
