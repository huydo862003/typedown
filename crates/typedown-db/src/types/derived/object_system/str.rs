use std::any::Any;
use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TdrTypeType};
use super::func::TdrFuncObj;
use crate::derived::evaluate::evaluate_node::evaluate_node;
use crate::derived::get_builtin_types::get_str_type;
use crate::types::{
  FuncSignature, HirValue, HirValueKind, InstResult, InterpolatedPart, TypeMember,
};
use crate::{Id, TypedownDatabase};

#[query_derived]
pub struct TdrStrType {}

impl TdrObjectLike for TdrStrType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

impl TdrTypeLike for TdrStrType {
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
      Box::new(TdrStrType::get(db)),
      sig,
      str_to_string,
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
      HirValueKind::Str(val) => Some(Box::new(TdrStrObj::new(db, val))),
      HirValueKind::Interpolated(parts) => {
        let mut val = String::new();
        for part in parts {
          match part {
            InterpolatedPart::Literal(lit) => val.push_str(&lit),
            InterpolatedPart::Expr(expr) => {
              let obj = evaluate_node(db, expr).value(db)?;
              let to_string_fn = obj.lookup_method(db, "to_string")?;
              let str_obj = to_string_fn.call(db, obj, vec![])?;
              let str_val = (str_obj.as_ref() as &dyn Any).downcast_ref::<TdrStrObj>()?;
              val.push_str(&str_val.value(db));
            }
          }
        }
        Some(Box::new(TdrStrObj::new(db, val)))
      }
      _ => None,
    }
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
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrStrType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

fn str_to_string(
  _db: &TypedownDatabase,
  this: Box<dyn TdrObjectLike>,
  _args: Vec<Box<dyn TdrObjectLike>>,
) -> Option<Box<dyn TdrObjectLike>> {
  // A string's to_string returns itself
  Some(this)
}
