use std::any::Any;
use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TdrTypeType};
use super::func::TdrFuncObj;
use super::str::{TdrStrObj, TdrStrType};
use crate::derived::get_builtin_types::get_math_type;
use crate::types::{FuncSignature, InstResult, TypeMember};
use crate::{Id, StableHash, StableHasher, TypedownDatabase};

#[query_derived]
pub struct TdrMathType {}

impl TdrObjectLike for TdrMathType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
  fn source_path(&self, _db: &TypedownDatabase) -> String {
    "@builtin::math".to_string()
  }

  fn as_type(&self) -> Option<Box<dyn TdrTypeLike>> {
    Some(Box::new(self.clone()))
  }
}

impl TdrTypeLike for TdrMathType {
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
      Box::new(TdrMathType::get(db)),
      sig,
      math_to_string,
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

  fn construct(
    &self,
    _db: &TypedownDatabase,
    args: Vec<Box<dyn TdrObjectLike>>,
  ) -> Option<Box<dyn TdrObjectLike>> {
    let arg = args.into_iter().next()?;
    (arg.as_ref() as &dyn Any).downcast_ref::<TdrMathObj>()?;
    Some(arg)
  }

  fn display_name(&self, _db: &TypedownDatabase) -> String {
    "math".to_string()
  }
}

impl TdrMathType {
  pub fn get(db: &TypedownDatabase) -> TdrMathType {
    get_math_type(db)
  }
}

#[query_derived]
pub struct TdrMathObj {
  pub value: String,
}

impl TdrObjectLike for TdrMathObj {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrMathType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
  fn source_path(&self, db: &TypedownDatabase) -> String {
    self.get_type(db).source_path(db)
  }
}

fn math_to_string(
  db: &TypedownDatabase,
  this: Box<dyn TdrObjectLike>,
  _args: Vec<Box<dyn TdrObjectLike>>,
) -> Option<Box<dyn TdrObjectLike>> {
  let math = (this.as_ref() as &dyn Any).downcast_ref::<TdrMathObj>()?;
  Some(Box::new(TdrStrObj::new(db, math.value(db))))
}

impl StableHash<TypedownDatabase> for TdrMathType {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.source_path(db).stable_hash(db, hasher);
  }
}

impl StableHash<TypedownDatabase> for TdrMathObj {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.value(db).stable_hash(db, hasher);
  }
}
