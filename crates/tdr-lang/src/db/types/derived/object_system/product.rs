use std::collections::HashMap;
use tdr_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike};
use super::func::TdrFuncObj;
use super::{TdrObjectEnum, TdrTypeEnum};
use crate::db::TypedownDatabase;
use crate::db::derived::evaluate::evaluate_node::evaluate_node;
use tdr_incremental::Id;
use tdr_types::either::Either;

use crate::db::derived::get_builtin_types::{get_bool_type, get_num_type, get_str_type};
use crate::db::types::{
  HirValue, InstResult, LiteralValue, MemberType, TypeMember, TypeMemberDescriptors,
};

/// Get the base TdrTypeEnum for a literal value
fn literal_base_type(db: &TypedownDatabase, lit: &LiteralValue) -> TdrTypeEnum {
  match lit {
    LiteralValue::Str(_) => get_str_type(db).into(),
    LiteralValue::Num(_) => get_num_type(db).into(),
    LiteralValue::Bool(_) => get_bool_type(db).into(),
  }
}

/// Check if actual can be assigned to expected
fn member_types_compatible(
  db: &TypedownDatabase,
  expected: &MemberType,
  actual: &MemberType,
) -> bool {
  match (expected, actual) {
    (MemberType::Simple(exp_type), MemberType::Simple(act_type)) => {
      exp_type.is_compatible_with(db, act_type)
    }
    (MemberType::Sum(exp_arms), MemberType::Sum(act_arms))
    | (MemberType::ListOfSum(exp_arms), MemberType::ListOfSum(act_arms))
    | (MemberType::DictOfSum(exp_arms), MemberType::DictOfSum(act_arms)) => {
      // Every arm in actual must be compatible with some arm in expected
      act_arms.iter().all(|act_arm| {
        let act_typ = act_arm.typ(db);
        exp_arms
          .iter()
          .any(|exp_arm| member_types_compatible(db, &exp_arm.typ(db), &act_typ))
      })
    }

    // Literal is a subtype of its base simple type
    (MemberType::Simple(exp_type), MemberType::Literal(act_val)) => {
      let base = literal_base_type(db, act_val);
      exp_type.is_compatible_with(db, &base)
    }

    // A simple/literal actual is assignable to a Sum expected if some arm matches.
    (MemberType::Sum(exp_arms), MemberType::Simple(_))
    | (MemberType::Sum(exp_arms), MemberType::Literal(_)) => exp_arms
      .iter()
      .any(|exp_arm| member_types_compatible(db, &exp_arm.typ(db), actual)),

    // ListOfSum is compatible with a Simple only if the simple is a list type whose elem type is compatible with some arm
    (MemberType::ListOfSum(exp_arms), MemberType::Simple(act_type)) => {
      match act_type.as_tdr_list_type() {
        Some(list) => match list.elem(db) {
          Some(elem) => {
            let elem_member = MemberType::Simple(elem);
            exp_arms
              .iter()
              .any(|exp_arm| member_types_compatible(db, &exp_arm.typ(db), &elem_member))
          }
          None => true,
        },
        None => false,
      }
    }

    // DictOfSum is compatible with a Simple if it's a dict type whose value matches some arm, or a product type whose every field's type matches some arm
    (MemberType::DictOfSum(exp_arms), MemberType::Simple(act_type)) => {
      if let Some(dict) = act_type.as_tdr_dict_type() {
        return match dict.value(db) {
          Some(value) => {
            let value_member = MemberType::Simple(value);
            exp_arms
              .iter()
              .any(|exp_arm| member_types_compatible(db, &exp_arm.typ(db), &value_member))
          }
          None => true,
        };
      }
      if let Some(product) = act_type.as_tdr_product_type() {
        return product.fields(db).values().all(|field_member| {
          let field_typ = field_member.typ(db);
          exp_arms
            .iter()
            .any(|exp_arm| member_types_compatible(db, &exp_arm.typ(db), &field_typ))
        });
      }
      false
    }

    // Sum assignable to simple if every arm is compatible with the simple
    (MemberType::Simple(_), MemberType::Sum(act_arms)) => act_arms
      .iter()
      .all(|act_arm| member_types_compatible(db, expected, &act_arm.typ(db))),

    // ListOfSum assignable to simple if the simple is a list and every arm is compatible with its elem
    (MemberType::Simple(exp_type), MemberType::ListOfSum(act_arms)) => {
      match exp_type.as_tdr_list_type() {
        Some(list) => match list.elem(db) {
          Some(elem) => {
            let elem_member = MemberType::Simple(elem);
            act_arms
              .iter()
              .all(|act_arm| member_types_compatible(db, &elem_member, &act_arm.typ(db)))
          }
          None => true,
        },
        None => false,
      }
    }
    // DictOfSum assignable to simple if the simple is a dict/product and every arm is compatible
    (MemberType::Simple(exp_type), MemberType::DictOfSum(act_arms)) => {
      if let Some(dict) = exp_type.as_tdr_dict_type() {
        return match dict.value(db) {
          Some(value) => {
            let value_member = MemberType::Simple(value);
            act_arms
              .iter()
              .all(|act_arm| member_types_compatible(db, &value_member, &act_arm.typ(db)))
          }
          None => true,
        };
      }

      if let Some(product) = exp_type.as_tdr_product_type() {
        // Every arm of the DictOfSum must be compatible with every field of the product
        return product.fields(db).values().all(|field_member| {
          let field_typ = field_member.typ(db);
          act_arms
            .iter()
            .all(|act_arm| member_types_compatible(db, &field_typ, &act_arm.typ(db)))
        });
      }
      false
    }

    // A string is not assignable to "foo", so (Literal, Simple) correctly falls through to false
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
  fn get_owned_field_type_member(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
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
      let actual_member = match actual.get_owned_field_type_member(db, field_name) {
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
    MemberType::ListOfSum(members) => {
      let inner = members
        .iter()
        .map(|m| member_type_display_name(db, &m.typ(db)))
        .collect::<Vec<_>>()
        .join(" | ");
      format!("list[{}]", inner)
    }
    MemberType::DictOfSum(members) => {
      let inner = members
        .iter()
        .map(|m| member_type_display_name(db, &m.typ(db)))
        .collect::<Vec<_>>()
        .join(" | ");
      format!("dict[{}]", inner)
    }
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
