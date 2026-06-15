use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeType, TdrTypeLike};
use super::func::TdrFuncType;
use crate::TypedownDatabase;
use crate::derived::get_builtin_types::get_link_type;
use crate::types::TypeMember;

#[query_derived]
pub struct TdrLinkType {
  pub schema: Option<Box<dyn TdrTypeLike>>,
}

impl TdrObjectLike for TdrLinkType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
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
    let mut iter = args.into_iter();
    Box::new(TdrLinkType::new(db, Some(iter.next().unwrap())))
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
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}
