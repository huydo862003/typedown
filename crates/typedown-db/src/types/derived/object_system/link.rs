use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TdrTypeType};
use super::func::TdrFuncType;
use crate::derived::get_builtin_types::get_link_type;
use crate::types::{HirValue, HirValueKind, TypeMember};
use crate::{Id, TypedownDatabase};

#[query_derived]
pub struct TdrLinkType {
  pub schema: Option<Box<dyn TdrTypeLike>>,
}

impl TdrObjectLike for TdrLinkType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

impl TdrTypeLike for TdrLinkType {
  fn arity(&self, db: &TypedownDatabase) -> usize {
    if self.schema(db).is_none() { 1 } else { 0 }
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
    let mut iter = args.into_iter();
    Box::new(TdrLinkType::new(db, Some(iter.next().unwrap())))
  }

  fn is_compatible_with(&self, db: &TypedownDatabase, actual: &dyn TdrTypeLike) -> bool {
    if self.as_id().0 != actual.as_id().0 {
      return false;
    }
    let self_args = self.get_type_args(db);
    if self_args.is_empty() {
      // Uninstantiated link: accept any link.
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
    self.schema(db).into_iter().collect()
  }

  fn construct(&self, db: &TypedownDatabase, hir: HirValue) -> Option<Box<dyn TdrObjectLike>> {
    match hir.kind(db) {
      HirValueKind::Str(val) => Some(Box::new(TdrLinkObj::new(db, val))),
      _ => None,
    }
  }

  fn display_name(&self, db: &TypedownDatabase) -> String {
    match self.schema(db) {
      Some(schema) => format!("link[{}]", schema.display_name(db)),
      None => "link".to_string(),
    }
  }
}

impl TdrLinkType {
  pub fn get(db: &TypedownDatabase) -> TdrLinkType {
    get_link_type(db)
  }
}

#[query_derived]
pub struct TdrLinkObj {
  pub target: String,
}

impl TdrObjectLike for TdrLinkObj {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrLinkType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}
