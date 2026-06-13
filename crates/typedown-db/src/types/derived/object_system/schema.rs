use std::collections::HashMap;

use typedown_macros::query_derived;

use crate::{
  TypedownDatabase,
  derived::get_builtin_types::{get_record_type, get_schema_type},
  types::{TdrFuncType, TdrObjectLike, TdrTypeLike, TypeMember},
};

#[query_derived]
pub struct TdrSchemaType {}

impl TdrObjectLike for TdrSchemaType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrSchemaType::get(db))
  }
  fn get_owned_fields(&self, db: &TypedownDatabase) -> HashMap<String, Box<dyn TdrObjectLike>> {
    HashMap::new()
  }
}

impl TdrTypeLike for TdrSchemaType {
  fn get_supertype(&self, db: &TypedownDatabase) -> Option<Box<dyn TdrTypeLike>> {
    Some(Box::new(get_record_type(db)))
  }

  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncType> {
    HashMap::new()
  }
  fn get_owned_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
    todo!()
  }
}

impl TdrSchemaType {
  pub fn get(db: &TypedownDatabase) -> TdrSchemaType {
    get_schema_type(db)
  }
}
