use std::collections::HashMap;
use tdr_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike};
use super::func::TdrFuncObj;
use super::{TdrObjectEnum, TdrTypeEnum};
use crate::db::TypedownDatabase;
use crate::db::derived::evaluate::evaluate_node::evaluate_node;
use tdr_incremental::Id;
use tdr_types::either::Either;

use crate::db::types::{HirValue, InstResult, MemberType, TypeMember, TypeMemberDescriptors};

fn member_types_compatible(
  db: &TypedownDatabase,
  expected: &MemberType,
  actual: &MemberType,
) -> bool {
  match (expected, actual) {
    (MemberType::Simple(exp_type), MemberType::Simple(act_type)) => {
      exp_type.is_compatible_with(db, act_type)
    }
    (MemberType::Sum(exp_arms), MemberType::Sum(act_arms)) => {
      // Two union types must have the same number of arms
      // each must be pairwise compatible
      if exp_arms.len() != act_arms.len() {
        return false;
      }
      exp_arms
        .iter()
        .zip(act_arms.iter())
        .all(|(exp_arm, act_arm)| member_types_compatible(db, &exp_arm.typ(db), &act_arm.typ(db)))
    }
    (MemberType::Literal(exp_val), MemberType::Literal(act_val)) => exp_val == act_val,
    (MemberType::Never, _) | (_, MemberType::Never) => false,
    _ => false,
  }
}

#[query_derived]
pub struct TdrProductType {
  pub name: Option<String>,
  pub metatype: TdrTypeEnum,
  pub fields: HashMap<String, TypeMember>,
}

impl TdrObjectLike for TdrProductType {
  fn get_type(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    self.metatype(db)
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<TdrObjectEnum> {
    None
  }
  fn source_path(&self, db: &TypedownDatabase) -> String {
    self.display_name(db)
  }
}

impl TdrTypeLike for TdrProductType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }
  fn get_supertype(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    TdrObjectType::get(db).into()
  }
  fn get_vtable(&self, _db: &TypedownDatabase) -> HashMap<String, TdrFuncObj> {
    HashMap::new()
  }
  fn get_owned_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
    self.fields(db).get(name).cloned()
  }
  fn instantiate(&self, db: &TypedownDatabase, args: Vec<TdrTypeEnum>) -> InstResult {
    assert_eq!(args.len(), self.arity(db), "arity mismatch");
    InstResult::new(db, (*self).into(), vec![])
  }
  fn get_type_args(&self, _db: &TypedownDatabase) -> Vec<TdrTypeEnum> {
    vec![]
  }
  fn is_compatible_with(&self, db: &TypedownDatabase, actual: &TdrTypeEnum) -> bool {
    if self.as_id().0 != actual.as_id().0 {
      return false;
    }
    let self_fields = self.fields(db);
    for (field_name, expected_member) in &self_fields {
      let is_required = !expected_member
        .descriptors(db)
        .contains(TypeMemberDescriptors::OPTIONAL);
      if !is_required {
        continue;
      }
      let actual_member = match actual.get_owned_field_type(db, field_name) {
        Some(member) => member,
        None => return false,
      };
      if !member_types_compatible(db, &expected_member.typ(db), &actual_member.typ(db)) {
        return false;
      }
    }
    true
  }
  fn construct(&self, db: &TypedownDatabase, args: Vec<TdrObjectEnum>) -> Option<TdrObjectEnum> {
    let arg = args.into_iter().next()?;
    let dict = arg.as_tdr_dict_obj()?;
    let fields = dict.entries(db);
    Some(TdrProductObj::new(db, (*self).into(), fields).into())
  }
  fn display_name(&self, db: &TypedownDatabase) -> String {
    if let Some(name) = self.name(db) {
      return name;
    }
    // Structural fallback for anonymous product types
    let fields = self.fields(db);
    if fields.is_empty() {
      return "{}".to_string();
    }
    let mut parts: Vec<String> = fields
      .iter()
      .map(|(name, member)| {
        let type_name = member_type_display_name(db, &member.typ(db));
        format!("{}: {}", name, type_name)
      })
      .collect();
    parts.sort();
    format!("{{ {} }}", parts.join(", "))
  }
}

pub(crate) fn member_type_display_name(db: &TypedownDatabase, member: &MemberType) -> String {
  match member {
    MemberType::Simple(typ) => typ.display_name(db),
    MemberType::Sum(members) => members
      .iter()
      .map(|m| member_type_display_name(db, &m.typ(db)))
      .collect::<Vec<_>>()
      .join(" | "),
    MemberType::Literal(val) => format!("{:?}", val),
    MemberType::Never => "never".to_string(),
  }
}

#[query_derived]
pub struct TdrProductObj {
  pub schema: TdrTypeEnum,
  pub fields: HashMap<String, Either<HirValue, TdrObjectEnum>>,
}

impl TdrObjectLike for TdrProductObj {
  fn get_type(&self, db: &TypedownDatabase) -> TdrTypeEnum {
    self.schema(db)
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<TdrObjectEnum> {
    match self.fields(db).get(key).cloned()? {
      Either::Left(hir) => evaluate_node(db, hir).value(db),
      Either::Right(obj) => Some(obj),
    }
  }
  fn source_path(&self, db: &TypedownDatabase) -> String {
    self.get_type(db).source_path(db)
  }
}
