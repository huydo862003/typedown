use std::collections::HashMap;

use typedown_macros::query_derived;

use crate::{
  TypedownDatabase,
  derived::get_builtin_types::{get_dict_type, get_schema_type},
  types::{TdrFuncType, TdrObjectLike, TdrTypeLike, TypeMember},
};

#[query_derived]
pub struct TdrSchemaType {}

impl TdrObjectLike for TdrSchemaType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrSchemaType::get(db))
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

impl TdrTypeLike for TdrSchemaType {
  fn arity(&self, db: &TypedownDatabase) -> usize {
    0
  }

  fn get_supertype(&self, db: &TypedownDatabase) -> Option<Box<dyn TdrTypeLike>> {
    Some(Box::new(get_dict_type(db)))
  }

  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncType> {
    HashMap::new()
  }
  fn get_owned_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
    todo!()
  }
  fn instantiate(
    &self,
    db: &TypedownDatabase,
    args: Vec<Box<dyn TdrTypeLike>>,
  ) -> Box<dyn TdrTypeLike> {
    Box::new(self.clone())
  }
}

impl TdrSchemaType {
  pub fn get(db: &TypedownDatabase) -> TdrSchemaType {
    get_schema_type(db)
  }
}
