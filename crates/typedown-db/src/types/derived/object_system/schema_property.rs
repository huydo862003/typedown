use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TdrTypeType};
use super::func::TdrFuncObj;
use crate::derived::get_builtin_types::{get_bool_type, get_schema_property_type, get_type_type};
use crate::types::{InstResult, MemberType, TypeMember, TypeMemberDescriptors};
use crate::{
  Decodable, Decoder, Encodable, Encoder, Id, StableHash, StableHasher, TypedownDatabase,
};

/// The type of a single property descriptor inside a schema's `properties` field.
/// Each property descriptor has:
///   - `type`: a type value (required)
///   - `optional`: a boolean (optional, defaults to false)
#[query_derived]
pub struct TdrSchemaPropertyType {}

impl TdrObjectLike for TdrSchemaPropertyType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
  fn source_path(&self, _db: &TypedownDatabase) -> String {
    "@builtin::schema_property".to_string()
  }

  fn as_type(&self) -> Option<Box<dyn TdrTypeLike>> {
    Some(Box::new(self.clone()))
  }
}

impl TdrTypeLike for TdrSchemaPropertyType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }
  fn get_supertype(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_vtable(&self, _db: &TypedownDatabase) -> HashMap<String, TdrFuncObj> {
    HashMap::new()
  }
  fn get_owned_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
    match name {
      "type" => Some(TypeMember::new(
        db,
        MemberType::Simple(Box::new(get_type_type(db))),
        TypeMemberDescriptors::empty(),
      )),
      "optional" => Some(TypeMember::new(
        db,
        MemberType::Simple(Box::new(get_bool_type(db))),
        TypeMemberDescriptors::OPTIONAL,
      )),
      _ => None,
    }
  }
  fn instantiate(&self, db: &TypedownDatabase, args: Vec<Box<dyn TdrTypeLike>>) -> InstResult {
    assert_eq!(args.len(), self.arity(db), "arity mismatch");
    InstResult::new(db, Box::new(self.clone()), vec![])
  }

  fn get_type_args(&self, _db: &TypedownDatabase) -> Vec<Box<dyn TdrTypeLike>> {
    vec![]
  }

  fn is_compatible_with(&self, _db: &TypedownDatabase, actual: &dyn TdrTypeLike) -> bool {
    self.as_id() == actual.as_id()
  }

  fn construct(
    &self,
    _db: &TypedownDatabase,
    _args: Vec<Box<dyn TdrObjectLike>>,
  ) -> Option<Box<dyn TdrObjectLike>> {
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

impl StableHash<TypedownDatabase> for TdrSchemaPropertyType {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.source_path(db).stable_hash(db, hasher);
  }
}

impl Encodable<TypedownDatabase> for TdrSchemaPropertyType {
  fn encode(&self, _encoder: &mut Encoder<TypedownDatabase>) {}
}

impl Decodable<TypedownDatabase> for TdrSchemaPropertyType {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    TdrSchemaPropertyType::get(decoder.db)
  }
}
