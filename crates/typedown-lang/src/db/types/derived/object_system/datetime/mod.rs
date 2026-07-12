mod utils;

use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrTypeLike, TdrTypeType};
use super::func::TdrFuncObj;
use super::native_fn::NativeFnKind;
use super::str::TdrStrType;
use super::{TdrObjectEnum, TdrTypeEnum};
use crate::db::TypedownDatabase;
use crate::db::derived::get_builtin_types::{get_date_type, get_datetime_type, get_time_type};
use crate::db::types::{FuncSignature, InstResult, TypeMember};
use typedown_incremental::Id;
pub(crate) use utils::{is_valid_iso_date, is_valid_iso_datetime, is_valid_iso_time};

// DateTime

#[query_derived]
pub struct TdrDateTimeType {}

impl TdrObjectLike for TdrDateTimeType {
  fn get_type(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrTypeType::get(db).into()
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<TdrObjectEnum> {
    None
  }
  fn source_path(&self, _db: &TypedownDatabase) -> String {
    "@builtin::datetime".to_string()
  }
}

impl TdrTypeLike for TdrDateTimeType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }
  fn get_supertype(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrStrType::get(db).into()
  }
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncObj> {
    let sig = FuncSignature::new(db, vec![], TdrStrType::get(db).into());
    let func_obj = TdrFuncObj::new(
      db,
      "to_string".to_string(),
      TdrDateTimeType::get(db).into(),
      sig,
      NativeFnKind::DateTimeToString,
    );
    HashMap::from([("to_string".to_string(), func_obj)])
  }
  fn get_owned_field_type(&self, _db: &TypedownDatabase, _name: &str) -> Option<TypeMember> {
    None
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
  fn construct(&self, db: &TypedownDatabase, args: Vec<TdrObjectEnum>) -> Option<TdrObjectEnum> {
    let arg = args.into_iter().next()?;
    let str_obj = arg.as_tdr_str_obj()?;
    let val = str_obj.value(db);
    if is_valid_iso_datetime(&val) {
      return Some(TdrDateTimeObj::new(db, val).into());
    }
    None
  }
  fn display_name(&self, _db: &TypedownDatabase) -> String {
    "datetime".to_string()
  }
}

impl TdrDateTimeType {
  pub fn get(db: &TypedownDatabase) -> TdrDateTimeType {
    get_datetime_type(db)
  }
}

#[query_derived]
pub struct TdrDateTimeObj {
  pub value: String,
}

impl TdrObjectLike for TdrDateTimeObj {
  fn get_type(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrDateTimeType::get(db).into()
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<TdrObjectEnum> {
    None
  }
  fn source_path(&self, db: &TypedownDatabase) -> String {
    self.get_type(db).source_path(db)
  }
  fn eq(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrDateTimeObj(other) = other {
      self.value(db) == other.value(db)
    } else {
      self.as_id() == other.as_id()
    }
  }
  fn lt(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrDateTimeObj(other) = other {
      self.value(db) < other.value(db)
    } else {
      self.as_id() < other.as_id()
    }
  }
  fn gt(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrDateTimeObj(other) = other {
      self.value(db) > other.value(db)
    } else {
      self.as_id() > other.as_id()
    }
  }
  fn le(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrDateTimeObj(other) = other {
      self.value(db) <= other.value(db)
    } else {
      self.as_id() <= other.as_id()
    }
  }
  fn ge(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrDateTimeObj(other) = other {
      self.value(db) >= other.value(db)
    } else {
      self.as_id() >= other.as_id()
    }
  }
}

// Date

#[query_derived]
pub struct TdrDateType {}

impl TdrObjectLike for TdrDateType {
  fn get_type(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrTypeType::get(db).into()
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<TdrObjectEnum> {
    None
  }
  fn source_path(&self, _db: &TypedownDatabase) -> String {
    "@builtin::date".to_string()
  }
}

impl TdrTypeLike for TdrDateType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }
  fn get_supertype(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrStrType::get(db).into()
  }
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncObj> {
    let sig = FuncSignature::new(db, vec![], TdrStrType::get(db).into());
    let func_obj = TdrFuncObj::new(
      db,
      "to_string".to_string(),
      TdrDateType::get(db).into(),
      sig,
      NativeFnKind::DateToString,
    );
    HashMap::from([("to_string".to_string(), func_obj)])
  }
  fn get_owned_field_type(&self, _db: &TypedownDatabase, _name: &str) -> Option<TypeMember> {
    None
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
  fn construct(&self, db: &TypedownDatabase, args: Vec<TdrObjectEnum>) -> Option<TdrObjectEnum> {
    let arg = args.into_iter().next()?;
    let str_obj = arg.as_tdr_str_obj()?;
    let val = str_obj.value(db);
    if is_valid_iso_date(&val) {
      return Some(TdrDateObj::new(db, val).into());
    }
    None
  }
  fn display_name(&self, _db: &TypedownDatabase) -> String {
    "date".to_string()
  }
}

impl TdrDateType {
  pub fn get(db: &TypedownDatabase) -> TdrDateType {
    get_date_type(db)
  }
}

#[query_derived]
pub struct TdrDateObj {
  pub value: String,
}

impl TdrObjectLike for TdrDateObj {
  fn get_type(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrDateType::get(db).into()
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<TdrObjectEnum> {
    None
  }
  fn source_path(&self, db: &TypedownDatabase) -> String {
    self.get_type(db).source_path(db)
  }
  fn eq(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrDateObj(other) = other {
      self.value(db) == other.value(db)
    } else {
      self.as_id() == other.as_id()
    }
  }
  fn lt(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrDateObj(other) = other {
      self.value(db) < other.value(db)
    } else {
      self.as_id() < other.as_id()
    }
  }
  fn gt(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrDateObj(other) = other {
      self.value(db) > other.value(db)
    } else {
      self.as_id() > other.as_id()
    }
  }
  fn le(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrDateObj(other) = other {
      self.value(db) <= other.value(db)
    } else {
      self.as_id() <= other.as_id()
    }
  }
  fn ge(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrDateObj(other) = other {
      self.value(db) >= other.value(db)
    } else {
      self.as_id() >= other.as_id()
    }
  }
}

// Time

#[query_derived]
pub struct TdrTimeType {}

impl TdrObjectLike for TdrTimeType {
  fn get_type(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrTypeType::get(db).into()
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<TdrObjectEnum> {
    None
  }
  fn source_path(&self, _db: &TypedownDatabase) -> String {
    "@builtin::time".to_string()
  }
}

impl TdrTypeLike for TdrTimeType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }
  fn get_supertype(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrStrType::get(db).into()
  }
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncObj> {
    let sig = FuncSignature::new(db, vec![], TdrStrType::get(db).into());
    let func_obj = TdrFuncObj::new(
      db,
      "to_string".to_string(),
      TdrTimeType::get(db).into(),
      sig,
      NativeFnKind::TimeToString,
    );
    HashMap::from([("to_string".to_string(), func_obj)])
  }
  fn get_owned_field_type(&self, _db: &TypedownDatabase, _name: &str) -> Option<TypeMember> {
    None
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
  fn construct(&self, db: &TypedownDatabase, args: Vec<TdrObjectEnum>) -> Option<TdrObjectEnum> {
    let arg = args.into_iter().next()?;
    let str_obj = arg.as_tdr_str_obj()?;
    let val = str_obj.value(db);
    if is_valid_iso_time(&val) {
      return Some(TdrTimeObj::new(db, val).into());
    }
    None
  }
  fn display_name(&self, _db: &TypedownDatabase) -> String {
    "time".to_string()
  }
}

impl TdrTimeType {
  pub fn get(db: &TypedownDatabase) -> TdrTimeType {
    get_time_type(db)
  }
}

#[query_derived]
pub struct TdrTimeObj {
  pub value: String,
}

impl TdrObjectLike for TdrTimeObj {
  fn get_type(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrTimeType::get(db).into()
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<TdrObjectEnum> {
    None
  }
  fn source_path(&self, db: &TypedownDatabase) -> String {
    self.get_type(db).source_path(db)
  }
  fn eq(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrTimeObj(other) = other {
      self.value(db) == other.value(db)
    } else {
      self.as_id() == other.as_id()
    }
  }
  fn lt(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrTimeObj(other) = other {
      self.value(db) < other.value(db)
    } else {
      self.as_id() < other.as_id()
    }
  }
  fn gt(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrTimeObj(other) = other {
      self.value(db) > other.value(db)
    } else {
      self.as_id() > other.as_id()
    }
  }
  fn le(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrTimeObj(other) = other {
      self.value(db) <= other.value(db)
    } else {
      self.as_id() <= other.as_id()
    }
  }
  fn ge(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrTimeObj(other) = other {
      self.value(db) >= other.value(db)
    } else {
      self.as_id() >= other.as_id()
    }
  }
}
