use std::collections::HashMap;
use typedown_incremental::Id;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TdrTypeType};
use super::func::TdrFuncObj;
use super::native_fn::NativeFnKind;
use super::str::TdrStrType;
use super::{TdrObjectEnum, TdrTypeEnum};
use crate::db::TypedownDatabase;
use crate::db::derived::get_builtin_types::get_math_type;
use crate::db::types::{FuncSignature, InstResult, TypeMember};

#[query_derived]
pub struct TdrMathType {}

impl TdrObjectLike for TdrMathType {
  fn get_type(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrTypeType::get(db).into()
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<TdrObjectEnum> {
    None
  }
  fn source_path(&self, _db: &TypedownDatabase) -> String {
    "@builtin::math".to_string()
  }
}

impl TdrTypeLike for TdrMathType {
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
      TdrMathType::get(db).into(),
      sig,
      NativeFnKind::MathToString,
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
  fn construct(&self, _db: &TypedownDatabase, args: Vec<TdrObjectEnum>) -> Option<TdrObjectEnum> {
    let arg = args.into_iter().next()?;
    arg.as_tdr_math_obj()?;
    Some(arg)
  }
  fn display_name(&self, _db: &TypedownDatabase) -> String {
    "math".to_string()
  }
}

impl TdrMathType {
  pub fn get(db: &TypedownDatabase) -> TdrMathType {
    get_math_type(db)
  }
}

#[query_derived]
pub struct TdrMathObj {
  pub value: String,
}

impl TdrObjectLike for TdrMathObj {
  fn get_type(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrMathType::get(db).into()
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<TdrObjectEnum> {
    None
  }
  fn source_path(&self, db: &TypedownDatabase) -> String {
    self.get_type(db).source_path(db)
  }
}
