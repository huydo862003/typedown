use std::any::Any;
use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TdrTypeType};
use super::func::TdrFuncObj;
use crate::derived::get_builtin_types::get_str_type;
use crate::types::{FuncSignature, InstResult, TypeMember};
use crate::{Id, StableHash, StableHasher, TypedownDatabase};

#[query_derived]
pub struct TdrStrType {}

impl TdrObjectLike for TdrStrType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
  fn source_path(&self, _db: &TypedownDatabase) -> String {
    "@builtin::string".to_string()
  }

  fn as_type(&self) -> Option<Box<dyn TdrTypeLike>> {
    Some(Box::new(self.clone()))
  }
}

impl TdrTypeLike for TdrStrType {
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
      Box::new(TdrStrType::get(db)),
      sig,
      str_to_string,
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

  fn is_compatible_with(&self, db: &TypedownDatabase, actual: &dyn TdrTypeLike) -> bool {
    if self.as_id() == actual.as_id() {
      return true;
    }
    // Accept subtypes of string (e.g. date, time, datetime) by walking the supertype chain
    let mut current = actual.get_supertype(db);
    loop {
      if self.as_id() == current.as_id() {
        return true;
      }
      let next = current.get_supertype(db);
      if next.as_id() == current.as_id() {
        return false;
      }
      current = next;
    }
  }

  fn construct(
    &self,
    _db: &TypedownDatabase,
    args: Vec<Box<dyn TdrObjectLike>>,
  ) -> Option<Box<dyn TdrObjectLike>> {
    let arg = args.into_iter().next()?;
    (arg.as_ref() as &dyn Any).downcast_ref::<TdrStrObj>()?;
    Some(arg)
  }

  fn display_name(&self, _db: &TypedownDatabase) -> String {
    "string".to_string()
  }
}

impl TdrStrType {
  pub fn get(db: &TypedownDatabase) -> TdrStrType {
    get_str_type(db)
  }
}

#[query_derived]
pub struct TdrStrObj {
  pub value: String,
}

impl TdrObjectLike for TdrStrObj {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrStrType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }

  fn source_path(&self, db: &TypedownDatabase) -> String {
    self.get_type(db).source_path(db)
  }

  fn eq(&self, db: &TypedownDatabase, other: &dyn TdrObjectLike) -> bool {
    match (other as &dyn Any).downcast_ref::<TdrStrObj>() {
      Some(other) => self.value(db) == other.value(db),
      None => self.as_id() == other.as_id(),
    }
  }
  fn lt(&self, db: &TypedownDatabase, other: &dyn TdrObjectLike) -> bool {
    match (other as &dyn Any).downcast_ref::<TdrStrObj>() {
      Some(other) => self.value(db) < other.value(db),
      None => self.as_id() < other.as_id(),
    }
  }
  fn gt(&self, db: &TypedownDatabase, other: &dyn TdrObjectLike) -> bool {
    match (other as &dyn Any).downcast_ref::<TdrStrObj>() {
      Some(other) => self.value(db) > other.value(db),
      None => self.as_id() > other.as_id(),
    }
  }
  fn le(&self, db: &TypedownDatabase, other: &dyn TdrObjectLike) -> bool {
    match (other as &dyn Any).downcast_ref::<TdrStrObj>() {
      Some(other) => self.value(db) <= other.value(db),
      None => self.as_id() <= other.as_id(),
    }
  }
  fn ge(&self, db: &TypedownDatabase, other: &dyn TdrObjectLike) -> bool {
    match (other as &dyn Any).downcast_ref::<TdrStrObj>() {
      Some(other) => self.value(db) >= other.value(db),
      None => self.as_id() >= other.as_id(),
    }
  }
}

fn str_to_string(
  _db: &TypedownDatabase,
  this: Box<dyn TdrObjectLike>,
  _args: Vec<Box<dyn TdrObjectLike>>,
) -> Option<Box<dyn TdrObjectLike>> {
  // A string's to_string returns itself
  Some(this)
}

impl StableHash<TypedownDatabase> for TdrStrType {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.source_path(db).stable_hash(db, hasher);
  }
}

impl StableHash<TypedownDatabase> for TdrStrObj {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.value(db).stable_hash(db, hasher);
  }
}
