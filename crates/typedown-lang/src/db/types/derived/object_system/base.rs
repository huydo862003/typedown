use std::collections::HashMap;

use ambassador::delegatable_trait;

use super::func::TdrFuncObj;
use super::native_fn::NativeFnKind;
use super::str::TdrStrType;
use super::{TdrObjectEnum, TdrTypeEnum};
use crate::db::derived::get_builtin_types::{get_object_type, get_str_type, get_type_type};
use crate::db::types::{FuncSignature, InstResult, MemberType, TypeMember, TypeMemberDescriptors};
use typedown_incremental::Id;
use typedown_macros::query_derived;

// Everything is an object
// This need not be object-safe
// We access via the enum, not via dyn trait
#[delegatable_trait]
pub trait TdrObjectLike: Id {
  fn get_type(&self, db: &crate::db::TypedownDatabase) -> TdrTypeEnum;

  fn lookup_method(&self, db: &crate::db::TypedownDatabase, key: &str) -> Option<TdrFuncObj> {
    let mut current = self.get_type(db);
    loop {
      if let Some(func_obj) = current.get_vtable(db).remove(key) {
        return Some(func_obj);
      }
      let supertype = current.get_supertype(db);
      if supertype.as_id() == current.as_id() {
        return None;
      }
      current = supertype;
    }
  }

  fn lookup_field(&self, db: &crate::db::TypedownDatabase, key: &str) -> Option<TdrObjectEnum> {
    if let Some(field) = self.get_owned_field(db, key) {
      return Some(field);
    }
    self.lookup_method(db, key).map(TdrObjectEnum::from)
  }

  fn get_owned_field(&self, db: &crate::db::TypedownDatabase, key: &str) -> Option<TdrObjectEnum>;

  fn source_path(&self, db: &crate::db::TypedownDatabase) -> String;

  fn eq(&self, _db: &crate::db::TypedownDatabase, other: &TdrObjectEnum) -> bool {
    self.as_id() == other.as_id()
  }

  fn lt(&self, _db: &crate::db::TypedownDatabase, other: &TdrObjectEnum) -> bool {
    self.as_id() < other.as_id()
  }

  fn gt(&self, _db: &crate::db::TypedownDatabase, other: &TdrObjectEnum) -> bool {
    self.as_id() > other.as_id()
  }

  fn le(&self, _db: &crate::db::TypedownDatabase, other: &TdrObjectEnum) -> bool {
    self.as_id() <= other.as_id()
  }

  fn ge(&self, _db: &crate::db::TypedownDatabase, other: &TdrObjectEnum) -> bool {
    self.as_id() >= other.as_id()
  }
}

fn get_builtin_field(db: &crate::db::TypedownDatabase, name: &str) -> Option<TypeMember> {
  match name {
    "_type" => Some(TypeMember::new(
      db,
      MemberType::Simple(TdrTypeType::get(db).into()),
      TypeMemberDescriptors::empty(),
    )),
    "_label" => Some(TypeMember::new(
      db,
      MemberType::Simple(get_str_type(db).into()),
      TypeMemberDescriptors::OPTIONAL,
    )),
    "_content" => Some(TypeMember::new(
      db,
      MemberType::Simple(get_str_type(db).into()),
      TypeMemberDescriptors::OPTIONAL,
    )),
    _ => None,
  }
}

// This need not be object-safe
// We access via the enum, not via dyn trait
#[delegatable_trait]
pub trait TdrTypeLike: TdrObjectLike {
  fn arity(&self, db: &crate::db::TypedownDatabase) -> usize;
  fn get_supertype(&self, db: &crate::db::TypedownDatabase) -> TdrTypeEnum;
  fn get_vtable(
    &self,
    db: &crate::db::TypedownDatabase,
  ) -> std::collections::HashMap<String, TdrFuncObj>;
  fn get_owned_field_type(
    &self,
    db: &crate::db::TypedownDatabase,
    name: &str,
  ) -> Option<crate::db::types::TypeMember>;

  fn instantiate(
    &self,
    db: &crate::db::TypedownDatabase,
    args: Vec<TdrTypeEnum>,
  ) -> crate::db::types::InstResult;

  fn is_compatible_with(&self, db: &crate::db::TypedownDatabase, actual: &TdrTypeEnum) -> bool;

  fn get_type_args(&self, db: &crate::db::TypedownDatabase) -> Vec<TdrTypeEnum>;

  fn display_name(&self, db: &crate::db::TypedownDatabase) -> String;

  fn construct(
    &self,
    db: &crate::db::TypedownDatabase,
    args: Vec<TdrObjectEnum>,
  ) -> Option<TdrObjectEnum>;

  fn get_field_type(
    &self,
    db: &crate::db::TypedownDatabase,
    name: &str,
  ) -> Option<crate::db::types::TypeMember> {
    if let Some(field) = get_builtin_field(db, name) {
      return Some(field);
    }
    if let Some(field) = self.get_owned_field_type(db, name) {
      return Some(field);
    }
    let supertype = self.get_supertype(db);
    if supertype.as_id() == self.as_id() {
      return None;
    }
    supertype.get_field_type(db, name)
  }

  fn lookup_instance_method(
    &self,
    db: &crate::db::TypedownDatabase,
    key: &str,
  ) -> Option<TdrFuncObj> {
    if let Some(func_obj) = self.get_vtable(db).remove(key) {
      return Some(func_obj);
    }
    let supertype = self.get_supertype(db);
    if supertype.as_id() == self.as_id() {
      return None;
    }
    supertype.lookup_instance_method(db, key)
  }

  fn lookup_field_type(&self, db: &crate::db::TypedownDatabase, name: &str) -> Option<TdrTypeEnum> {
    if let Some(member) = self.get_field_type(db, name) {
      if let MemberType::Simple(typ) = member.typ(db) {
        return Some(typ);
      }
    }
    self
      .lookup_instance_method(db, name)
      .map(|func_obj| func_obj.get_type(db))
  }
}

/// The metatype is the type of all types.
/// It's an instance of itself and the type of every type.
#[query_derived]
pub struct TdrTypeType {}

impl TdrObjectLike for TdrTypeType {
  fn get_type(&self, db: &crate::db::TypedownDatabase) -> TdrTypeEnum {
    TdrTypeType::get(db).into()
  }
  fn get_owned_field(
    &self,
    _db: &crate::db::TypedownDatabase,
    _key: &str,
  ) -> Option<TdrObjectEnum> {
    None
  }
  fn source_path(&self, _db: &crate::db::TypedownDatabase) -> String {
    "@builtin::type".to_string()
  }
}

impl TdrTypeLike for TdrTypeType {
  fn arity(&self, _db: &crate::db::TypedownDatabase) -> usize {
    0
  }
  fn get_supertype(&self, db: &crate::db::TypedownDatabase) -> TdrTypeEnum {
    TdrObjectType::get(db).into()
  }
  fn get_vtable(
    &self,
    _db: &crate::db::TypedownDatabase,
  ) -> std::collections::HashMap<String, TdrFuncObj> {
    HashMap::new()
  }
  fn get_owned_field_type(
    &self,
    _db: &crate::db::TypedownDatabase,
    _name: &str,
  ) -> Option<TypeMember> {
    None
  }
  fn instantiate(&self, db: &crate::db::TypedownDatabase, _args: Vec<TdrTypeEnum>) -> InstResult {
    InstResult::new(db, self.clone().into(), vec![])
  }
  fn get_type_args(&self, _db: &crate::db::TypedownDatabase) -> Vec<TdrTypeEnum> {
    vec![]
  }
  fn is_compatible_with(&self, _db: &crate::db::TypedownDatabase, actual: &TdrTypeEnum) -> bool {
    self.as_id() == actual.as_id()
  }
  fn construct(
    &self,
    _db: &crate::db::TypedownDatabase,
    _args: Vec<TdrObjectEnum>,
  ) -> Option<TdrObjectEnum> {
    // HIR-level construction (ident/mapping paths) lives in utils.rs
    None
  }
  fn display_name(&self, _db: &crate::db::TypedownDatabase) -> String {
    "type".to_string()
  }
}

impl TdrTypeType {
  pub fn get(db: &crate::db::TypedownDatabase) -> TdrTypeType {
    get_type_type(db)
  }
}

/// The base type for all objects in TDR
#[query_derived]
pub struct TdrObjectType {}

impl TdrObjectLike for TdrObjectType {
  fn get_type(&self, db: &crate::db::TypedownDatabase) -> TdrTypeEnum {
    TdrTypeType::get(db).into()
  }
  fn get_owned_field(
    &self,
    _db: &crate::db::TypedownDatabase,
    _key: &str,
  ) -> Option<TdrObjectEnum> {
    None
  }
  fn source_path(&self, _db: &crate::db::TypedownDatabase) -> String {
    "@builtin::object".to_string()
  }
}

impl TdrTypeLike for TdrObjectType {
  fn arity(&self, _db: &crate::db::TypedownDatabase) -> usize {
    0
  }
  fn get_supertype(&self, db: &crate::db::TypedownDatabase) -> TdrTypeEnum {
    TdrObjectType::get(db).into()
  }
  fn get_vtable(
    &self,
    db: &crate::db::TypedownDatabase,
  ) -> std::collections::HashMap<String, TdrFuncObj> {
    let sig = FuncSignature::new(db, vec![], TdrStrType::get(db).into());
    let func_obj = TdrFuncObj::new(
      db,
      "to_string".to_string(),
      get_object_type(db).into(),
      sig,
      NativeFnKind::ObjectToString,
    );
    HashMap::from([("to_string".to_string(), func_obj)])
  }
  fn get_owned_field_type(
    &self,
    _db: &crate::db::TypedownDatabase,
    _name: &str,
  ) -> Option<TypeMember> {
    None
  }
  fn instantiate(&self, db: &crate::db::TypedownDatabase, _args: Vec<TdrTypeEnum>) -> InstResult {
    InstResult::new(db, self.clone().into(), vec![])
  }
  fn get_type_args(&self, _db: &crate::db::TypedownDatabase) -> Vec<TdrTypeEnum> {
    vec![]
  }
  fn is_compatible_with(&self, _db: &crate::db::TypedownDatabase, actual: &TdrTypeEnum) -> bool {
    self.as_id() == actual.as_id()
  }
  fn construct(
    &self,
    _db: &crate::db::TypedownDatabase,
    args: Vec<TdrObjectEnum>,
  ) -> Option<TdrObjectEnum> {
    args.into_iter().next()
  }
  fn display_name(&self, _db: &crate::db::TypedownDatabase) -> String {
    "object".to_string()
  }
}

impl TdrObjectType {
  pub fn get(db: &crate::db::TypedownDatabase) -> TdrObjectType {
    get_object_type(db)
  }
}
