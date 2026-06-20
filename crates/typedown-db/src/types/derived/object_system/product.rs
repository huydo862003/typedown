use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike};
use super::func::TdrFuncType;
use crate::derived::evaluate::evaluate_resource::construct_from_hir;
use crate::{Id, TypedownDatabase};

use crate::types::{HirValue, HirValueKind, InstResult, MemberType, TypeMember};

fn member_type_compatible(
  db: &TypedownDatabase,
  expected: &MemberType,
  actual: &MemberType,
) -> bool {
  match (expected, actual) {
    (MemberType::Simple(exp_type), MemberType::Simple(act_type)) => {
      exp_type.is_compatible_with(db, act_type.as_ref())
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
        .all(|(exp_arm, act_arm)| member_type_compatible(db, &exp_arm.typ(db), &act_arm.typ(db)))
    }
    _ => false,
  }
}

#[query_derived]
pub struct TdrProductType {
  pub name: Option<String>,
  pub metatype: Box<dyn TdrTypeLike>,
  pub fields: HashMap<String, TypeMember>,
}

impl TdrObjectLike for TdrProductType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    self.metatype(db)
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

impl TdrTypeLike for TdrProductType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }

  fn get_supertype(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }

  fn get_vtable(&self, _db: &TypedownDatabase) -> HashMap<String, TdrFuncType> {
    HashMap::new()
  }

  fn get_owned_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
    self.fields(db).get(name).cloned()
  }

  fn instantiate(&self, db: &TypedownDatabase, args: Vec<Box<dyn TdrTypeLike>>) -> InstResult {
    assert_eq!(args.len(), self.arity(db), "arity mismatch");
    InstResult::new(db, Box::new(self.clone()), vec![])
  }

  fn get_type_args(&self, _db: &TypedownDatabase) -> Vec<Box<dyn TdrTypeLike>> {
    vec![]
  }

  fn is_compatible_with(&self, db: &TypedownDatabase, actual: &dyn TdrTypeLike) -> bool {
    if self.as_id().0 != actual.as_id().0 {
      return false;
    }
    let self_fields = self.fields(db);
    for (field_name, expected_member) in &self_fields {
      let is_required = !expected_member
        .descriptors(db)
        .contains(crate::types::TypeMemberDescriptors::OPTIONAL);
      if !is_required {
        continue;
      }
      let actual_member = match actual.get_owned_field_type(db, field_name) {
        Some(member) => member,
        None => return false,
      };
      if !member_type_compatible(db, &expected_member.typ(db), &actual_member.typ(db)) {
        return false;
      }
    }
    true
  }

  fn construct(&self, db: &TypedownDatabase, hir: HirValue) -> Option<Box<dyn TdrObjectLike>> {
    match hir.kind(db) {
      HirValueKind::Mapping(entries) => {
        let mut fields = HashMap::new();
        for (key, value_hir) in entries {
          if key == "_type" {
            continue;
          }
          fields.insert(key, value_hir);
        }
        Some(Box::new(TdrProductObj::new(
          db,
          Box::new(self.clone()) as Box<dyn TdrTypeLike>,
          fields,
        )))
      }
      _ => None,
    }
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
  }
}

#[query_derived]
pub struct TdrProductObj {
  pub schema: Box<dyn TdrTypeLike>,
  pub fields: HashMap<String, HirValue>,
}

impl TdrObjectLike for TdrProductObj {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    self.schema(db)
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    let hir = self.fields(db).get(key).cloned()?;
    construct_from_hir(db, hir)
  }
}
