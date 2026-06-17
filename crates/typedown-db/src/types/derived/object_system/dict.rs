use std::any::Any;
use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TdrTypeType};
use super::func::TdrFuncType;
use crate::{Id, TypedownDatabase};
use crate::derived::get_builtin_types::get_dict_type;
use crate::types::TypeMember;

#[query_derived]
pub struct TdrDictType {
  pub key: Option<Box<dyn TdrTypeLike>>,
  pub value: Option<Box<dyn TdrTypeLike>>,
}

impl TdrObjectLike for TdrDictType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

impl TdrTypeLike for TdrDictType {
  fn arity(&self, db: &TypedownDatabase) -> usize {
    [self.key(db).is_none(), self.value(db).is_none()]
      .iter()
      .filter(|&&absent| absent)
      .count()
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
    let key = iter.next().unwrap();
    let value = iter.next().unwrap();
    Box::new(TdrDictType::new(db, Some(key), Some(value)))
  }

  fn is_compatible_with(&self, db: &TypedownDatabase, actual: &dyn TdrTypeLike) -> bool {
    if self.as_id().0 != actual.as_id().0 {
      return false;
    }
    let self_args = self.get_type_args(db);
    if self_args.is_empty() {
      // Uninstantiated dict: accept any dict.
      return true;
    }
    let actual_args = actual.get_type_args(db);
    if actual_args.is_empty() {
      return false;
    }
    self_args
      .iter()
      .zip(actual_args.iter())
      .all(|(s, a)| s.is_compatible_with(db, a.as_ref()))
  }

  fn get_type_args(&self, db: &TypedownDatabase) -> Vec<Box<dyn TdrTypeLike>> {
    match (self.key(db), self.value(db)) {
      (Some(key), Some(value)) => vec![key, value],
      _ => vec![],
    }
  }
}

impl TdrDictType {
  pub fn get(db: &TypedownDatabase) -> TdrDictType {
    get_dict_type(db)
  }
}

#[query_derived]
pub struct TdrDictObj {
  pub entries: HashMap<String, Box<dyn TdrObjectLike>>,
}

impl TdrObjectLike for TdrDictObj {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrDictType::get(db))
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}
