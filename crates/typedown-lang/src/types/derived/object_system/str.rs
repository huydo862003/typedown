use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TdrTypeType};
use super::func::TdrFuncObj;
use super::native_fn::NativeFnKind;
use super::{TdrObjectEnum, TdrTypeEnum};
use crate::TypedownDatabase;
use crate::derived::get_builtin_types::get_str_type;
use crate::types::{FuncSignature, InstResult, TypeMember};
use typedown_incremental::Id;

#[query_derived]
pub struct TdrStrType {}

impl TdrObjectLike for TdrStrType {
  fn get_type(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrTypeType::get(db).into()
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<TdrObjectEnum> {
    None
  }
  fn source_path(&self, _db: &TypedownDatabase) -> String {
    "@builtin::string".to_string()
  }
}

impl TdrTypeLike for TdrStrType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }
  fn get_supertype(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrObjectType::get(db).into()
  }
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncObj> {
    let sig = FuncSignature::new(db, vec![], TdrStrType::get(db).into());
    let func_obj = TdrFuncObj::new(
      db,
      "to_string".to_string(),
      TdrStrType::get(db).into(),
      sig,
      NativeFnKind::StrToString,
    );
    HashMap::from([("to_string".to_string(), func_obj)])
  }
  fn get_owned_field_type(&self, _db: &TypedownDatabase, _name: &str) -> Option<TypeMember> {
    None
  }
  fn instantiate(&self, db: &TypedownDatabase, args: Vec<TdrTypeEnum>) -> InstResult {
    assert_eq!(args.len(), self.arity(db), "arity mismatch");
    InstResult::new(db, self.clone().into(), vec![])
  }
  fn get_type_args(&self, _db: &TypedownDatabase) -> Vec<TdrTypeEnum> {
    vec![]
  }
  fn is_compatible_with(&self, db: &TypedownDatabase, actual: &TdrTypeEnum) -> bool {
    if self.as_id() == actual.as_id() {
      return true;
    }
    // Accept subtypes of string (e.g. date, time, datetime) by walking the supertype chain
    let mut current = actual.get_supertype(db);
    loop {
      if self.as_id() == current.as_id() {
        return true;
      }
      let next = current.get_supertype(db);
      if next.as_id() == current.as_id() {
        return false;
      }
      current = next;
    }
  }
  fn construct(&self, _db: &TypedownDatabase, args: Vec<TdrObjectEnum>) -> Option<TdrObjectEnum> {
    let arg = args.into_iter().next()?;
    arg.as_tdr_str_obj()?;
    Some(arg)
  }
  fn display_name(&self, _db: &TypedownDatabase) -> String {
    "string".to_string()
  }
}

impl TdrStrType {
  pub fn get(db: &TypedownDatabase) -> TdrStrType {
    get_str_type(db)
  }
}

#[query_derived]
pub struct TdrStrObj {
  pub value: String,
}

impl TdrObjectLike for TdrStrObj {
  fn get_type(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrStrType::get(db).into()
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<TdrObjectEnum> {
    None
  }
  fn source_path(&self, db: &TypedownDatabase) -> String {
    self.get_type(db).source_path(db)
  }
  fn eq(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrStrObj(other) = other {
      self.value(db) == other.value(db)
    } else {
      self.as_id() == other.as_id()
    }
  }
  fn lt(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrStrObj(other) = other {
      self.value(db) < other.value(db)
    } else {
      self.as_id() < other.as_id()
    }
  }
  fn gt(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrStrObj(other) = other {
      self.value(db) > other.value(db)
    } else {
      self.as_id() > other.as_id()
    }
  }
  fn le(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrStrObj(other) = other {
      self.value(db) <= other.value(db)
    } else {
      self.as_id() <= other.as_id()
    }
  }
  fn ge(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrStrObj(other) = other {
      self.value(db) >= other.value(db)
    } else {
      self.as_id() >= other.as_id()
    }
  }
}
