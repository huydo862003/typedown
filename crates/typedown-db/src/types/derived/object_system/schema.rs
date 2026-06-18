use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrTypeLike, TdrTypeType};
use super::dict::TdrDictType;
use super::func::TdrFuncType;
use crate::derived::get_builtin_types::{get_schema_property_type, get_schema_type, get_str_type};
use crate::types::{MemberType, TypeMember, TypeMemberDescriptors};
use crate::{Id, TypedownDatabase};

// Schema type is actually a kind
// and its a subtype of the "type" kind
#[query_derived]
pub struct TdrSchemaType {}

impl TdrObjectLike for TdrSchemaType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

impl TdrTypeLike for TdrSchemaType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }

  fn get_supertype(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }

  fn get_vtable(&self, _db: &TypedownDatabase) -> HashMap<String, TdrFuncType> {
    HashMap::new()
  }

  fn get_owned_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
    match name {
      "properties" => {
        let properties_type = TdrDictType::new(
          db,
          Some(Box::new(get_str_type(db))),
          Some(Box::new(get_schema_property_type(db))),
        );
        Some(TypeMember::new(
          db,
          MemberType::Simple(Box::new(properties_type)),
          TypeMemberDescriptors::empty(),
        ))
      }
      _ => None,
    }
  }

  fn instantiate(
    &self,
    _db: &TypedownDatabase,
    _args: Vec<Box<dyn TdrTypeLike>>,
  ) -> Box<dyn TdrTypeLike> {
    Box::new(self.clone())
  }

  fn get_type_args(&self, _db: &TypedownDatabase) -> Vec<Box<dyn TdrTypeLike>> {
    vec![]
  }

  fn is_compatible_with(&self, _db: &TypedownDatabase, actual: &dyn TdrTypeLike) -> bool {
    self.as_id() == actual.as_id()
  }

  fn display_name(&self, _db: &TypedownDatabase) -> String {
    "Schema".to_string()
  }
}

impl TdrSchemaType {
  pub fn get(db: &TypedownDatabase) -> TdrSchemaType {
    get_schema_type(db)
  }
}
