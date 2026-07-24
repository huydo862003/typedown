use std::collections::HashMap;

use tdr_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TdrTypeType};
use super::func::TdrFuncObj;
use super::str::TdrStrType;
use super::{TdrObjectEnum, TdrStrObj, TdrTypeEnum};
use crate::db::TypedownDatabase;
use crate::db::derived::get_builtin_types::get_blob_type;
use crate::db::types::{AssetKind, InstResult, MemberType, TypeMember, TypeMemberDescriptors};
use tdr_incremental::Id;

#[query_derived]
pub struct TdrBlobType {}

impl TdrObjectLike for TdrBlobType {
  fn get_type(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrTypeType::get(db).into()
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<TdrObjectEnum> {
    None
  }
  fn source_path(&self, _db: &TypedownDatabase) -> String {
    "@builtin::blob".to_string()
  }
}

impl TdrTypeLike for TdrBlobType {
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
    let str_type: TdrTypeEnum = TdrStrType::get(db).into();
    match name {
      "format" => Some(TypeMember::new(
        db,
        MemberType::Simple(str_type),
        TypeMemberDescriptors::empty(),
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
    self.as_id() == actual.as_id()
  }
  fn construct(&self, _db: &TypedownDatabase, _args: Vec<TdrObjectEnum>) -> Option<TdrObjectEnum> {
    None
  }
  fn display_name(&self, _db: &TypedownDatabase) -> String {
    "blob".to_string()
  }
}

impl TdrBlobType {
  pub fn get(db: &TypedownDatabase) -> TdrBlobType {
    get_blob_type(db)
  }
}

#[query_derived]
pub struct TdrBlobObj {
  asset_kind: AssetKind,
}

impl TdrObjectLike for TdrBlobObj {
  fn get_type(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrBlobType::get(db).into()
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<TdrObjectEnum> {
    match key {
      "format" => {
        let format_str = match self.asset_kind(db) {
          AssetKind::Pdf => "pdf",
          AssetKind::Svg => "svg",
          AssetKind::Png => "png",
          AssetKind::Jpg => "jpg",
          AssetKind::Webp => "webp",
          AssetKind::UnknownBinary => "unknown",
        };
        Some(TdrStrObj::new(db, format_str.to_string()).into())
      }
      _ => None,
    }
  }
  fn source_path(&self, db: &TypedownDatabase) -> String {
    self.get_type(db).source_path(db)
  }
}
