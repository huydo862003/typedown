use std::any::Any;
use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TdrTypeType};
use super::func::TdrFuncObj;
use crate::derived::evaluate::utils::construct_from_hir;
use crate::derived::get_builtin_types::get_dict_type;
use crate::types::{HirValue, HirValueKind, InstResult, TdrProductType, TypeMember};
use crate::{Id, TypedownDatabase};

#[query_derived]
pub struct TdrDictType {
  pub key: Option<Box<dyn TdrTypeLike>>,
  pub value: Option<Box<dyn TdrTypeLike>>,
}

impl TdrObjectLike for TdrDictType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

impl TdrTypeLike for TdrDictType {
  fn arity(&self, db: &TypedownDatabase) -> usize {
    [self.key(db).is_none(), self.value(db).is_none()]
      .iter()
      .filter(|&&absent| absent)
      .count()
  }

  fn get_supertype(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }

  fn get_vtable(&self, _db: &TypedownDatabase) -> HashMap<String, TdrFuncObj> {
    HashMap::new()
  }

  fn get_owned_field_type(&self, _db: &TypedownDatabase, _name: &str) -> Option<TypeMember> {
    None
  }
  fn instantiate(&self, db: &TypedownDatabase, args: Vec<Box<dyn TdrTypeLike>>) -> InstResult {
    assert_eq!(args.len(), self.arity(db), "arity mismatch");
    let mut iter = args.into_iter();
    let key = iter.next().unwrap();
    let value = iter.next().unwrap();
    InstResult::new(
      db,
      Box::new(TdrDictType::new(db, Some(key), Some(value))),
      vec![],
    )
  }

  fn is_compatible_with(&self, db: &TypedownDatabase, actual: &dyn TdrTypeLike) -> bool {
    // Structural compatibility: accept a product type if all field values match the dict's value type
    if let Some(product) = (actual as &dyn Any).downcast_ref::<TdrProductType>() {
      let value_type = match self.value(db) {
        Some(vt) => vt,
        None => return true,
      };
      return product
        .fields(db)
        .values()
        .all(|member| match member.typ(db) {
          crate::types::MemberType::Simple(field_type) => {
            value_type.is_compatible_with(db, field_type.as_ref())
          }
          _ => false,
        });
    }

    // Dict-to-dict compatibility
    if self.as_id().0 != actual.as_id().0 {
      return false;
    }
    let self_args = self.get_type_args(db);
    if self_args.is_empty() {
      return true;
    }
    let actual_args = actual.get_type_args(db);
    if actual_args.is_empty() {
      return false;
    }
    self_args
      .iter()
      .zip(actual_args.iter())
      .all(|(s, a)| s.is_compatible_with(db, a.as_ref()))
  }

  fn get_type_args(&self, db: &TypedownDatabase) -> Vec<Box<dyn TdrTypeLike>> {
    match (self.key(db), self.value(db)) {
      (Some(key), Some(value)) => vec![key, value],
      _ => vec![],
    }
  }

  fn construct(&self, db: &TypedownDatabase, hir: HirValue) -> Option<Box<dyn TdrObjectLike>> {
    match hir.kind(db) {
      HirValueKind::Mapping(entries) => {
        let map = entries.into_iter().collect();
        Some(Box::new(TdrDictObj::new(db, map)))
      }
      _ => None,
    }
  }

  fn display_name(&self, db: &TypedownDatabase) -> String {
    match (self.key(db), self.value(db)) {
      (Some(key), Some(value)) => {
        format!("dict[{}, {}]", key.display_name(db), value.display_name(db))
      }
      _ => "dict".to_string(),
    }
  }
}

impl TdrDictType {
  pub fn get(db: &TypedownDatabase) -> TdrDictType {
    get_dict_type(db)
  }
}

#[query_derived]
pub struct TdrDictObj {
  pub entries: HashMap<String, HirValue>,
}

impl TdrObjectLike for TdrDictObj {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrDictType::get(db))
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    let hir = self.entries(db).get(key).cloned()?;
    construct_from_hir(db, hir)
  }
}
