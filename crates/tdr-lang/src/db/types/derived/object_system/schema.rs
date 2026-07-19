use std::collections::HashMap;
use tdr_incremental::Id;
use tdr_macros::query_derived;

use super::base::{TdrObjectLike, TdrTypeLike, TdrTypeType};
use super::dict::TdrDictType;
use super::func::TdrFuncObj;
use super::{TdrObjectEnum, TdrProductType, TdrTypeEnum};
use crate::db::TypedownDatabase;
use crate::db::derived::evaluate::evaluate_node::evaluate_node;
use crate::db::derived::get_builtin_types::{
  get_schema_property_type, get_schema_type, get_str_type,
};
use crate::db::types::{InstResult, MemberType, TypeMember, TypeMemberDescriptors};
use tdr_types::either::Either;

// Schema type is actually a kind
// and its a subtype of the "type" kind
#[query_derived]
pub struct TdrSchemaType {}

impl TdrObjectLike for TdrSchemaType {
  fn get_type(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrTypeType::get(db).into()
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<TdrObjectEnum> {
    None
  }
  fn source_path(&self, _db: &TypedownDatabase) -> String {
    "@builtin::schema".to_string()
  }
}

impl TdrTypeLike for TdrSchemaType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }
  fn get_supertype(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrTypeType::get(db).into()
  }
  fn get_vtable(&self, _db: &TypedownDatabase) -> HashMap<String, TdrFuncObj> {
    HashMap::new()
  }
  fn get_owned_field_type_member(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
    match name {
      "properties" => {
        let properties_type = TdrDictType::new(
          db,
          Some(get_str_type(db).into()),
          Some(get_schema_property_type(db).into()),
        );
        Some(TypeMember::new(
          db,
          MemberType::Simple(properties_type.into()),
          TypeMemberDescriptors::empty(),
        ))
      }
      _ => None,
    }
  }
  fn instantiate(&self, db: &TypedownDatabase, _args: Vec<TdrTypeEnum>) -> InstResult {
    InstResult::new(db, (*self).into(), vec![])
  }
  fn get_type_args(&self, _db: &TypedownDatabase) -> Vec<TdrTypeEnum> {
    vec![]
  }
  fn is_compatible_with(&self, _db: &TypedownDatabase, actual: &TdrTypeEnum) -> bool {
    self.as_id() == actual.as_id()
  }
  fn construct(&self, db: &TypedownDatabase, args: Vec<TdrObjectEnum>) -> Option<TdrObjectEnum> {
    let arg = args.into_iter().next()?;
    let dict = arg.as_tdr_dict_obj()?;
    let mut fields = HashMap::new();
    for (name, entry) in dict.entries(db) {
      let obj = match entry {
        Either::Left(hir) => evaluate_node(db, hir).value(db)?,
        Either::Right(obj) => obj,
      };
      let typ = obj.as_type()?;
      fields.insert(
        name,
        TypeMember::new(db, MemberType::Simple(typ), TypeMemberDescriptors::empty()),
      );
    }
    Some(TdrProductType::new(db, None, TdrSchemaType::get(db).into(), fields).into())
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
