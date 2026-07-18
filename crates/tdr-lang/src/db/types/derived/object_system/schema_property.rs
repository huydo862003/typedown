use std::collections::HashMap;
use tdr_incremental::Id;
use tdr_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TdrTypeType};
use super::func::TdrFuncObj;
use super::{TdrObjectEnum, TdrTypeEnum};
use crate::db::TypedownDatabase;
use crate::db::derived::get_builtin_types::{
  get_bool_type, get_schema_property_type, get_type_type,
};
use crate::db::types::{InstResult, MemberType, TypeMember, TypeMemberDescriptors};

/// The type of a single property descriptor inside a schema's `properties` field.
/// Each property descriptor has:
///   - `type`: a type value (required)
///   - `optional`: a boolean (optional, defaults to false)
#[query_derived]
pub struct TdrSchemaPropertyType {}

impl TdrObjectLike for TdrSchemaPropertyType {
  fn get_type(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrTypeType::get(db).into()
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<TdrObjectEnum> {
    None
  }
  fn source_path(&self, _db: &TypedownDatabase) -> String {
    "@builtin::schema_property".to_string()
  }
}

impl TdrTypeLike for TdrSchemaPropertyType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }
  fn get_supertype(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrObjectType::get(db).into()
  }
  fn get_vtable(&self, _db: &TypedownDatabase) -> HashMap<String, TdrFuncObj> {
    HashMap::new()
  }
  fn get_owned_field_type_member(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
    match name {
      "type" => Some(TypeMember::new(
        db,
        MemberType::Simple(get_type_type(db).into()),
        TypeMemberDescriptors::empty(),
      )),
      "optional" => Some(TypeMember::new(
        db,
        MemberType::Simple(get_bool_type(db).into()),
        TypeMemberDescriptors::OPTIONAL,
      )),
      _ => None,
    }
  }
  fn instantiate(&self, db: &TypedownDatabase, args: Vec<TdrTypeEnum>) -> InstResult {
    assert_eq!(args.len(), self.arity(db), "arity mismatch");
    InstResult::new(db, (*self).into(), vec![])
  }
  fn get_type_args(&self, _db: &TypedownDatabase) -> Vec<TdrTypeEnum> {
    vec![]
  }
  fn is_compatible_with(&self, _db: &TypedownDatabase, actual: &TdrTypeEnum) -> bool {
    if self.as_id() == actual.as_id() {
      return true;
    }
    // FIXME: Currently, the type system is not sophisticated enough
    // But schema_property is an opaque type anyways...
    // We don't actually provide any validation here.

    // Property descriptors are structurally validated by
    // evaluate_type::resolve_property_descriptor
    if actual.is_tdr_product_type() {
      return true;
    }
    false
  }
  fn construct(&self, _db: &TypedownDatabase, _args: Vec<TdrObjectEnum>) -> Option<TdrObjectEnum> {
    None
  }
  fn display_name(&self, _db: &TypedownDatabase) -> String {
    "SchemaProperty".to_string()
  }
}

impl TdrSchemaPropertyType {
  pub fn get(db: &TypedownDatabase) -> TdrSchemaPropertyType {
    get_schema_property_type(db)
  }
}
