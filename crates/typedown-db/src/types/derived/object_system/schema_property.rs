use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TdrTypeType};
use super::func::TdrFuncType;
use crate::TypedownDatabase;
use crate::derived::get_builtin_types::{get_bool_type, get_schema_property_type, get_type_type};
use crate::types::{MemberType, TypeMember, TypeMemberDescriptors};

/// The type of a single property descriptor inside a schema's `properties` field.
/// Each property descriptor has:
///   - `type`: a type value (required)
///   - `required`: a boolean (optional)
#[query_derived]
pub struct TdrSchemaPropertyType {}

impl TdrObjectLike for TdrSchemaPropertyType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

impl TdrTypeLike for TdrSchemaPropertyType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }
  fn get_supertype(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_vtable(&self, _db: &TypedownDatabase) -> HashMap<String, TdrFuncType> {
    HashMap::new()
  }
  fn get_owned_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
    match name {
      "type" => Some(TypeMember::new(
        db,
        MemberType::Simple(Box::new(get_type_type(db))),
        TypeMemberDescriptors::empty(),
      )),
      "required" => Some(TypeMember::new(
        db,
        MemberType::Simple(Box::new(get_bool_type(db))),
        TypeMemberDescriptors::OPTIONAL,
      )),
      _ => None,
    }
  }
  fn instantiate(
    &self,
    db: &TypedownDatabase,
    args: Vec<Box<dyn TdrTypeLike>>,
  ) -> Box<dyn TdrTypeLike> {
    assert_eq!(args.len(), self.arity(db), "arity mismatch");
    Box::new(self.clone())
  }
}

impl TdrSchemaPropertyType {
  pub fn get(db: &TypedownDatabase) -> TdrSchemaPropertyType {
    get_schema_property_type(db)
  }
}
