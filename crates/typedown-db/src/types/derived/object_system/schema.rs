use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrTypeLike, TdrTypeType};
use super::dict::TdrDictType;
use super::func::TdrFuncObj;
use crate::derived::evaluate::evaluate_type::resolve_property_descriptor;
use crate::derived::get_builtin_types::{get_schema_property_type, get_schema_type, get_str_type};
use crate::types::{
  HirValue, HirValueKind, InstResult, MemberType, TdrProductType, TypeMember, TypeMemberDescriptors,
};
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
  fn as_type(&self) -> Option<Box<dyn TdrTypeLike>> {
    Some(Box::new(self.clone()))
  }
}

impl TdrTypeLike for TdrSchemaType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }

  fn get_supertype(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }

  fn get_vtable(&self, _db: &TypedownDatabase) -> HashMap<String, TdrFuncObj> {
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

  fn instantiate(&self, db: &TypedownDatabase, _args: Vec<Box<dyn TdrTypeLike>>) -> InstResult {
    InstResult::new(db, Box::new(self.clone()), vec![])
  }

  fn get_type_args(&self, _db: &TypedownDatabase) -> Vec<Box<dyn TdrTypeLike>> {
    vec![]
  }

  fn is_compatible_with(&self, _db: &TypedownDatabase, actual: &dyn TdrTypeLike) -> bool {
    self.as_id() == actual.as_id()
  }

  fn construct(&self, db: &TypedownDatabase, hir: HirValue) -> Option<Box<dyn TdrObjectLike>> {
    // Build a TdrProductType from the properties field
    let entries = match hir.kind(db) {
      HirValueKind::Mapping(entries) => entries,
      _ => return None,
    };

    let properties_entries = match entries.iter().find(|(key, _)| key == "properties") {
      Some((_, props_hir)) => match props_hir.kind(db) {
        HirValueKind::Mapping(entries) => entries,
        _ => return None,
      },
      None => {
        return Some(Box::new(TdrProductType::new(
          db,
          None,
          Box::new(TdrSchemaType::get(db)),
          HashMap::new(),
        )));
      }
    };

    let mut fields = HashMap::new();
    for (prop_name, prop_hir) in properties_entries {
      if let Some((member_type, descriptors)) =
        resolve_property_descriptor(db, prop_hir, &mut vec![])
      {
        fields.insert(
          prop_name.clone(),
          TypeMember::new(db, member_type, descriptors),
        );
      }
    }

    Some(Box::new(TdrProductType::new(
      db,
      None,
      Box::new(TdrSchemaType::get(db)),
      fields,
    )))
  }

  fn display_name(&self, _db: &TypedownDatabase) -> String {
    "schema".to_string()
  }
}

impl TdrSchemaType {
  pub fn get(db: &TypedownDatabase) -> TdrSchemaType {
    get_schema_type(db)
  }
}
