use std::collections::HashMap;
use tdr_macros::query_derived;
use tdr_types::either::Either;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TdrTypeType};
use super::func::TdrFuncObj;
use super::{TdrObjectEnum, TdrTypeEnum};
use crate::db::TypedownDatabase;
use crate::db::derived::evaluate::evaluate_node::evaluate_node;
use crate::db::derived::get_builtin_types::get_dict_type;
use crate::db::types::{HirValue, InstResult, MemberType, TypeMember};
use crate::db::utils::typecheck::member_types_compatible;
use tdr_incremental::Id;

#[query_derived]
pub struct TdrDictType {
  pub key: Option<TdrTypeEnum>,
  pub value: Option<TdrTypeEnum>,
}

impl TdrObjectLike for TdrDictType {
  fn get_type(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrTypeType::get(db).into()
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<TdrObjectEnum> {
    None
  }
  fn source_path(&self, db: &TypedownDatabase) -> String {
    match (self.key(db), self.value(db)) {
      (Some(key), Some(value)) => format!(
        "@builtin::dict[{}, {}]",
        key.source_path(db),
        value.source_path(db)
      ),
      _ => "@builtin::dict".to_string(),
    }
  }
}

impl TdrTypeLike for TdrDictType {
  fn arity(&self, db: &TypedownDatabase) -> usize {
    [self.key(db).is_none(), self.value(db).is_none()]
      .iter()
      .filter(|&&absent| absent)
      .count()
  }
  fn get_supertype(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrObjectType::get(db).into()
  }
  fn get_vtable(&self, _db: &TypedownDatabase) -> HashMap<String, TdrFuncObj> {
    HashMap::new()
  }
  fn get_owned_field_type_member(&self, _db: &TypedownDatabase, _name: &str) -> Option<TypeMember> {
    None
  }
  fn instantiate(&self, db: &TypedownDatabase, args: Vec<TdrTypeEnum>) -> InstResult {
    assert_eq!(args.len(), self.arity(db), "arity mismatch");
    let mut iter = args.into_iter();
    let key = iter.next().unwrap();
    let value = iter.next().unwrap();
    InstResult::new(
      db,
      TdrDictType::new(db, Some(key), Some(value)).into(),
      vec![],
    )
  }
  fn is_compatible_with(&self, db: &TypedownDatabase, actual: &TdrTypeEnum) -> bool {
    if let TdrTypeEnum::TdrProductType(product) = actual {
      let value_type = match self.value(db) {
        Some(vt) => vt,
        None => return true,
      };
      return product.fields(db).values().all(|member| {
        let value_member = MemberType::Simple(value_type.clone());
        member_types_compatible(db, &value_member, &member.typ(db))
      });
    }

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
      .all(|(s, a)| s.is_compatible_with(db, a))
  }
  fn get_type_args(&self, db: &TypedownDatabase) -> Vec<TdrTypeEnum> {
    match (self.key(db), self.value(db)) {
      (Some(key), Some(value)) => vec![key, value],
      _ => vec![],
    }
  }
  fn construct(&self, db: &TypedownDatabase, args: Vec<TdrObjectEnum>) -> Option<TdrObjectEnum> {
    let mut entries = HashMap::new();
    for arg in args {
      let pair = arg.as_tdr_list_obj()?;
      if pair.len(db) != 2 {
        return None;
      }
      let key_obj = pair.get(db, 0)?;
      let key_str = key_obj.as_tdr_str_obj()?.value(db);
      let val = pair.get(db, 1)?;
      entries.insert(key_str, Either::Right(val));
    }
    Some(TdrDictObj::new(db, entries).into())
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
  pub entries: HashMap<String, Either<HirValue, TdrObjectEnum>>,
}

impl TdrObjectLike for TdrDictObj {
  fn get_type(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrDictType::get(db).into()
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<TdrObjectEnum> {
    match self.entries(db).get(key).cloned()? {
      Either::Left(hir) => evaluate_node(db, hir).value(db),
      Either::Right(obj) => Some(obj),
    }
  }
  fn source_path(&self, db: &TypedownDatabase) -> String {
    self.get_type(db).source_path(db)
  }
}
