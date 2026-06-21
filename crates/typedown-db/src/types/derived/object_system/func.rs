use std::any::Any;
use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TdrTypeType};
use super::str::{TdrStrObj, TdrStrType};
use crate::derived::get_builtin_types::get_func_type;
use crate::types::{FuncSignature, HirValue, InstResult, TypeMember};
use crate::{Id, TypedownDatabase};

pub type NativeFn = fn(
  &TypedownDatabase,
  Box<dyn TdrObjectLike>,
  Vec<Box<dyn TdrObjectLike>>,
) -> Option<Box<dyn TdrObjectLike>>;

#[query_derived]
pub struct TdrFuncType {
  #[id]
  pub signature: FuncSignature,
}

impl TdrObjectLike for TdrFuncType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
  fn as_type(&self) -> Option<Box<dyn TdrTypeLike>> {
    Some(Box::new(self.clone()))
  }
}

impl TdrTypeLike for TdrFuncType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }

  fn get_supertype(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncObj> {
    let sig = FuncSignature::new(db, vec![], Box::new(TdrStrType::get(db)));
    let func_obj = TdrFuncObj::new(
      db,
      "to_string".to_string(),
      Box::new(self.clone()),
      sig,
      func_to_string,
    );
    HashMap::from([("to_string".to_string(), func_obj)])
  }
  fn get_owned_field_type(&self, _db: &TypedownDatabase, _name: &str) -> Option<TypeMember> {
    None
  }
  fn instantiate(&self, db: &TypedownDatabase, args: Vec<Box<dyn TdrTypeLike>>) -> InstResult {
    assert_eq!(args.len(), self.arity(db), "arity mismatch");
    InstResult::new(db, Box::new(self.clone()), vec![])
  }

  fn get_type_args(&self, _db: &TypedownDatabase) -> Vec<Box<dyn TdrTypeLike>> {
    vec![]
  }

  fn is_compatible_with(&self, _db: &TypedownDatabase, actual: &dyn TdrTypeLike) -> bool {
    self.as_id() == actual.as_id()
  }

  fn construct(&self, _db: &TypedownDatabase, _hir: HirValue) -> Option<Box<dyn TdrObjectLike>> {
    None
  }

  fn display_name(&self, _db: &TypedownDatabase) -> String {
    "function".to_string()
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
  #[id]
  pub name: String,
  #[id]
  pub typ: Box<dyn TdrTypeLike>,
  #[id]
  pub signature: FuncSignature,
  #[skip]
  pub func: NativeFn,
}

impl TdrFuncObj {
  pub fn call(
    &self,
    db: &TypedownDatabase,
    this: Box<dyn TdrObjectLike>,
    args: Vec<Box<dyn TdrObjectLike>>,
  ) -> Option<Box<dyn TdrObjectLike>> {
    (self.func(db))(db, this, args)
  }
}

impl TdrObjectLike for TdrFuncObj {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(get_func_type(db, self.signature(db)))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

fn func_to_string(
  db: &TypedownDatabase,
  this: Box<dyn TdrObjectLike>,
  _args: Vec<Box<dyn TdrObjectLike>>,
) -> Option<Box<dyn TdrObjectLike>> {
  let func = (this.as_ref() as &dyn Any).downcast_ref::<TdrFuncObj>()?;
  Some(Box::new(TdrStrObj::new(db, func.name(db))))
}
