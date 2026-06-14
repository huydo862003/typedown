use crate::Id;
use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike};
use crate::types::TypeMember;
use super::func::TdrFuncType;
use crate::TypedownDatabase;
use crate::derived::get_builtin_types::get_record_type;

#[query_derived]
pub struct TdrRecordType {}

impl TdrObjectLike for TdrRecordType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

impl TdrTypeLike for TdrRecordType {
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

impl TdrRecordType {
  pub fn get(db: &TypedownDatabase) -> TdrRecordType {
    get_record_type(db)
  }
}

#[query_derived]
pub struct TdrRecordObj {
  pub entries: HashMap<String, Box<dyn TdrObjectLike>>,
}

impl TdrObjectLike for TdrRecordObj {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrRecordType::get(db))
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}
