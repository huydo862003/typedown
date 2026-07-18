use std::collections::HashMap;
use tdr_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TdrTypeType};
use super::func::TdrFuncObj;
use super::native_fn::NativeFnKind;
use super::str::TdrStrType;
use super::{TdrObjectEnum, TdrTypeEnum};
use crate::db::TypedownDatabase;
use crate::db::derived::get_builtin_types::get_num_type;
use crate::db::types::{FuncSignature, InstResult, TypeMember};
use tdr_incremental::Id;

#[query_derived]
pub struct TdrNumType {}

impl TdrObjectLike for TdrNumType {
  fn get_type(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrTypeType::get(db).into()
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<TdrObjectEnum> {
    None
  }
  fn source_path(&self, _db: &TypedownDatabase) -> String {
    "@builtin::number".to_string()
  }
}

impl TdrTypeLike for TdrNumType {
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
      TdrNumType::get(db).into(),
      sig,
      NativeFnKind::NumToString,
    );
    HashMap::from([("to_string".to_string(), func_obj)])
  }
  fn get_owned_field_type_member(&self, _db: &TypedownDatabase, _name: &str) -> Option<TypeMember> {
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
  fn construct(&self, _db: &TypedownDatabase, args: Vec<TdrObjectEnum>) -> Option<TdrObjectEnum> {
    let arg = args.into_iter().next()?;
    arg.as_tdr_num_obj()?;
    Some(arg)
  }
  fn display_name(&self, _db: &TypedownDatabase) -> String {
    "number".to_string()
  }
}

impl TdrNumType {
  pub fn get(db: &TypedownDatabase) -> TdrNumType {
    get_num_type(db)
  }
}

#[query_derived]
pub struct TdrNumObj {
  pub value: f64,
}

impl TdrObjectLike for TdrNumObj {
  fn get_type(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrNumType::get(db).into()
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<TdrObjectEnum> {
    None
  }
  fn source_path(&self, db: &TypedownDatabase) -> String {
    self.get_type(db).source_path(db)
  }
  fn eq(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrNumObj(other) = other {
      self.value(db) == other.value(db)
    } else {
      self.as_id() == other.as_id()
    }
  }
  fn lt(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrNumObj(other) = other {
      self.value(db) < other.value(db)
    } else {
      self.as_id() < other.as_id()
    }
  }
  fn gt(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrNumObj(other) = other {
      self.value(db) > other.value(db)
    } else {
      self.as_id() > other.as_id()
    }
  }
  fn le(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrNumObj(other) = other {
      self.value(db) <= other.value(db)
    } else {
      self.as_id() <= other.as_id()
    }
  }
  fn ge(&self, db: &TypedownDatabase, other: &TdrObjectEnum) -> bool {
    if let TdrObjectEnum::TdrNumObj(other) = other {
      self.value(db) >= other.value(db)
    } else {
      self.as_id() >= other.as_id()
    }
  }
}
