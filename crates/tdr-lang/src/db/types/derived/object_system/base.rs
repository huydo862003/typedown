//! We follow a simple system for supertypes
//! - Only owned fields can be accessed via the object
//! - Methods can be inheritted via supertypes

use std::collections::HashMap;

use ambassador::delegatable_trait;

use super::func::TdrFuncObj;
use super::native_fn::NativeFnKind;
use super::str::TdrStrType;
use super::{TdrObjectEnum, TdrTypeEnum};
use crate::db::derived::get_builtin_types::{get_object_type, get_type_type};
use crate::db::types::{FuncSignature, InstResult, MemberType, TypeMember, TypeMemberDescriptors};
use tdr_incremental::Id;
use tdr_macros::query_derived;

// Everything is an object
// This need not be object-safe
// We access via the enum, not via dyn trait
#[delegatable_trait]
pub trait TdrObjectLike: Id {
  fn get_type(&self, db: &::tdr_lang::db::TypedownDatabase) -> TdrTypeEnum;

  fn lookup_method(&self, db: &::tdr_lang::db::TypedownDatabase, key: &str) -> Option<TdrFuncObj> {
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

  fn lookup_field(
    &self,
    db: &::tdr_lang::db::TypedownDatabase,
    key: &str,
  ) -> Option<TdrObjectEnum> {
    if let Some(field) = self.get_owned_field(db, key) {
      return Some(field);
    }
    self.lookup_method(db, key).map(TdrObjectEnum::from)
  }

  fn get_owned_field(
    &self,
    db: &::tdr_lang::db::TypedownDatabase,
    key: &str,
  ) -> Option<TdrObjectEnum>;

  fn source_path(&self, db: &::tdr_lang::db::TypedownDatabase) -> String;

  fn eq(&self, _db: &::tdr_lang::db::TypedownDatabase, other: &TdrObjectEnum) -> bool {
    self.as_id() == other.as_id()
  }

  fn lt(&self, _db: &::tdr_lang::db::TypedownDatabase, other: &TdrObjectEnum) -> bool {
    self.as_id() < other.as_id()
  }

  fn gt(&self, _db: &::tdr_lang::db::TypedownDatabase, other: &TdrObjectEnum) -> bool {
    self.as_id() > other.as_id()
  }

  fn le(&self, _db: &::tdr_lang::db::TypedownDatabase, other: &TdrObjectEnum) -> bool {
    self.as_id() <= other.as_id()
  }

  fn ge(&self, _db: &::tdr_lang::db::TypedownDatabase, other: &TdrObjectEnum) -> bool {
    self.as_id() >= other.as_id()
  }
}

// This need not be object-safe
// We access via the enum, not via dyn trait
#[delegatable_trait]
pub trait TdrTypeLike: TdrObjectLike {
  fn arity(&self, db: &::tdr_lang::db::TypedownDatabase) -> usize;
  fn get_supertype(&self, db: &::tdr_lang::db::TypedownDatabase) -> TdrTypeEnum;
  fn get_vtable(
    &self,
    db: &::tdr_lang::db::TypedownDatabase,
  ) -> std::collections::HashMap<String, TdrFuncObj>;
  fn get_owned_field_type_member(
    &self,
    db: &::tdr_lang::db::TypedownDatabase,
    name: &str,
  ) -> Option<::tdr_lang::db::types::TypeMember>;
  fn lookup_field_type_member(
    &self,
    db: &::tdr_lang::db::TypedownDatabase,
    name: &str,
  ) -> Option<::tdr_lang::db::types::TypeMember> {
    self.get_owned_field_type_member(db, name).or_else(|| {
      Some(TypeMember::new(
        db,
        MemberType::Simple(self.lookup_method(db, name)?.get_type(db)),
        TypeMemberDescriptors::empty(),
      ))
    })
  }

  fn instantiate(
    &self,
    db: &::tdr_lang::db::TypedownDatabase,
    args: Vec<TdrTypeEnum>,
  ) -> ::tdr_lang::db::types::InstResult;

  fn is_compatible_with(&self, db: &::tdr_lang::db::TypedownDatabase, actual: &TdrTypeEnum)
  -> bool;

  fn get_type_args(&self, db: &::tdr_lang::db::TypedownDatabase) -> Vec<TdrTypeEnum>;

  fn display_name(&self, db: &::tdr_lang::db::TypedownDatabase) -> String;

  fn construct(
    &self,
    db: &::tdr_lang::db::TypedownDatabase,
    args: Vec<TdrObjectEnum>,
  ) -> Option<TdrObjectEnum>;

  fn lookup_instance_method(
    &self,
    db: &::tdr_lang::db::TypedownDatabase,
    key: &str,
  ) -> Option<TdrFuncObj> {
    if let Some(func_obj) = self.get_vtable(db).get(key) {
      return Some(func_obj.clone());
    }
    let supertype = self.get_supertype(db);
    if supertype.as_id() == self.as_id() {
      return None;
    }
    supertype.lookup_instance_method(db, key)
  }
}

/// The metatype is the type of all types.
/// It's an instance of itself and the type of every type.
#[query_derived]
pub struct TdrTypeType {}

impl TdrObjectLike for TdrTypeType {
  fn get_type(&self, db: &::tdr_lang::db::TypedownDatabase) -> TdrTypeEnum {
    TdrTypeType::get(db).into()
  }
  fn get_owned_field(
    &self,
    _db: &::tdr_lang::db::TypedownDatabase,
    _key: &str,
  ) -> Option<TdrObjectEnum> {
    None
  }
  fn source_path(&self, _db: &::tdr_lang::db::TypedownDatabase) -> String {
    "@builtin::type".to_string()
  }
}

impl TdrTypeLike for TdrTypeType {
  fn arity(&self, _db: &::tdr_lang::db::TypedownDatabase) -> usize {
    0
  }
  fn get_supertype(&self, db: &::tdr_lang::db::TypedownDatabase) -> TdrTypeEnum {
    TdrObjectType::get(db).into()
  }
  fn get_vtable(
    &self,
    _db: &::tdr_lang::db::TypedownDatabase,
  ) -> std::collections::HashMap<String, TdrFuncObj> {
    HashMap::new()
  }
  fn get_owned_field_type_member(
    &self,
    _db: &::tdr_lang::db::TypedownDatabase,
    _name: &str,
  ) -> Option<TypeMember> {
    None
  }
  fn instantiate(
    &self,
    db: &::tdr_lang::db::TypedownDatabase,
    _args: Vec<TdrTypeEnum>,
  ) -> InstResult {
    InstResult::new(db, (*self).into(), vec![])
  }
  fn get_type_args(&self, _db: &::tdr_lang::db::TypedownDatabase) -> Vec<TdrTypeEnum> {
    vec![]
  }
  fn is_compatible_with(
    &self,
    _db: &::tdr_lang::db::TypedownDatabase,
    actual: &TdrTypeEnum,
  ) -> bool {
    self.as_id() == actual.as_id()
  }
  fn construct(
    &self,
    _db: &::tdr_lang::db::TypedownDatabase,
    _args: Vec<TdrObjectEnum>,
  ) -> Option<TdrObjectEnum> {
    None
  }
  fn display_name(&self, _db: &::tdr_lang::db::TypedownDatabase) -> String {
    "type".to_string()
  }
}

impl TdrTypeType {
  pub fn get(db: &::tdr_lang::db::TypedownDatabase) -> TdrTypeType {
    get_type_type(db)
  }
}

/// The base type for all objects in TDR
#[query_derived]
pub struct TdrObjectType {}

impl TdrObjectLike for TdrObjectType {
  fn get_type(&self, db: &::tdr_lang::db::TypedownDatabase) -> TdrTypeEnum {
    TdrTypeType::get(db).into()
  }
  fn get_owned_field(
    &self,
    _db: &::tdr_lang::db::TypedownDatabase,
    _key: &str,
  ) -> Option<TdrObjectEnum> {
    None
  }
  fn source_path(&self, _db: &::tdr_lang::db::TypedownDatabase) -> String {
    "@builtin::object".to_string()
  }
}

impl TdrTypeLike for TdrObjectType {
  fn arity(&self, _db: &::tdr_lang::db::TypedownDatabase) -> usize {
    0
  }
  fn get_supertype(&self, db: &::tdr_lang::db::TypedownDatabase) -> TdrTypeEnum {
    TdrObjectType::get(db).into()
  }
  fn get_vtable(
    &self,
    db: &::tdr_lang::db::TypedownDatabase,
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
  fn get_owned_field_type_member(
    &self,
    _db: &::tdr_lang::db::TypedownDatabase,
    _name: &str,
  ) -> Option<TypeMember> {
    None
  }
  fn instantiate(
    &self,
    db: &::tdr_lang::db::TypedownDatabase,
    _args: Vec<TdrTypeEnum>,
  ) -> InstResult {
    InstResult::new(db, (*self).into(), vec![])
  }
  fn get_type_args(&self, _db: &::tdr_lang::db::TypedownDatabase) -> Vec<TdrTypeEnum> {
    vec![]
  }
  fn is_compatible_with(
    &self,
    _db: &::tdr_lang::db::TypedownDatabase,
    actual: &TdrTypeEnum,
  ) -> bool {
    self.as_id() == actual.as_id()
  }
  fn construct(
    &self,
    _db: &::tdr_lang::db::TypedownDatabase,
    args: Vec<TdrObjectEnum>,
  ) -> Option<TdrObjectEnum> {
    args.into_iter().next()
  }
  fn display_name(&self, _db: &::tdr_lang::db::TypedownDatabase) -> String {
    "object".to_string()
  }
}

impl TdrObjectType {
  pub fn get(db: &::tdr_lang::db::TypedownDatabase) -> TdrObjectType {
    get_object_type(db)
  }
}
