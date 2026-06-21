use std::any::Any;
use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TdrTypeType};
use super::func::TdrFuncObj;
use super::str::{TdrStrObj, TdrStrType};
use crate::derived::get_builtin_types::get_num_type;
use crate::types::{FuncSignature, HirValue, HirValueKind, InstResult, TypeMember};
use crate::{Id, TypedownDatabase};

#[query_derived]
pub struct TdrNumType {}

impl TdrObjectLike for TdrNumType {
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

impl TdrTypeLike for TdrNumType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }

  fn get_supertype(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncObj> {
    let sig = FuncSignature::new(db, vec![], Box::new(TdrStrType::get(db)));
    let func_obj = TdrFuncObj::new(
      db,
      "to_string".to_string(),
      Box::new(TdrNumType::get(db)),
      sig,
      num_to_string,
    );
    HashMap::from([("to_string".to_string(), func_obj)])
  }
  fn get_owned_field_type(&self, _db: &TypedownDatabase, _name: &str) -> Option<TypeMember> {
    None
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

  fn construct(&self, db: &TypedownDatabase, hir: HirValue) -> Option<Box<dyn TdrObjectLike>> {
    match hir.kind(db) {
      HirValueKind::Num(val) => {
        let num: f64 = val.parse().unwrap_or(0.0);
        Some(Box::new(TdrNumObj::new(db, num)))
      }
      _ => None,
    }
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
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrNumType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

fn num_to_string(
  db: &TypedownDatabase,
  this: Box<dyn TdrObjectLike>,
  _args: Vec<Box<dyn TdrObjectLike>>,
) -> Option<Box<dyn TdrObjectLike>> {
  let num = (this.as_ref() as &dyn Any).downcast_ref::<TdrNumObj>()?;
  let value = num.value(db);
  let s = if value.fract() == 0.0 {
    format!("{}", value as i64)
  } else {
    format!("{}", value)
  };
  Some(Box::new(TdrStrObj::new(db, s)))
}
