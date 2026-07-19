//! Shared type compatibility utilities for typechecking

use crate::db::TypedownDatabase;
use crate::db::derived::get_builtin_types::{get_bool_type, get_num_type, get_str_type};
use crate::db::types::{
  HirValue, HirValueKind, LiteralValue, MemberType, TdrTypeEnum, TdrTypeLike,
};

/// Get the base TdrTypeEnum for a literal value
pub fn literal_base_type(db: &TypedownDatabase, lit: &LiteralValue) -> TdrTypeEnum {
  match lit {
    LiteralValue::Str(_) => get_str_type(db).into(),
    LiteralValue::Num(_) => get_num_type(db).into(),
    LiteralValue::Bool(_) => get_bool_type(db).into(),
  }
}

/// Check if actual MemberType can be assigned to expected MemberType
pub fn member_types_compatible(
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

/// Check if a value's actual type matches the expected member type
pub fn value_matches_member_type(
  db: &TypedownDatabase,
  expected: &MemberType,
  actual: &TdrTypeEnum,
  value_hir: HirValue,
) -> bool {
  match expected {
    MemberType::Simple(exp_type) => exp_type.is_compatible_with(db, actual),
    MemberType::Sum(members) => members
      .iter()
      .any(|member| value_matches_member_type(db, &member.typ(db), actual, value_hir)),
    MemberType::ListOfSum(members) => {
      // Actual must be a list type, and its elem must match some arm
      match actual.as_tdr_list_type() {
        Some(list) => match list.elem(db) {
          Some(elem) => members
            .iter()
            .any(|member| value_matches_member_type(db, &member.typ(db), &elem, value_hir)),
          None => true,
        },
        None => false,
      }
    }
    MemberType::DictOfSum(members) => {
      // Actual must be a dict or product type, and its values must match some arm
      if let Some(dict) = actual.as_tdr_dict_type() {
        return match dict.value(db) {
          Some(value) => members
            .iter()
            .any(|member| value_matches_member_type(db, &member.typ(db), &value, value_hir)),
          None => true,
        };
      }
      if let Some(product) = actual.as_tdr_product_type() {
        return product
          .fields(db)
          .values()
          .all(|field_member| match field_member.typ(db) {
            MemberType::Simple(field_type) => members
              .iter()
              .any(|member| value_matches_member_type(db, &member.typ(db), &field_type, value_hir)),
            _ => false,
          });
      }
      false
    }
    MemberType::Literal(literal) => match (literal, value_hir.kind(db)) {
      (LiteralValue::Str(expected_val), HirValueKind::Str(actual_val)) => {
        *expected_val == actual_val
      }
      (LiteralValue::Num(expected_val), HirValueKind::Num(actual_val)) => {
        *expected_val == actual_val
      }
      (LiteralValue::Bool(expected_val), HirValueKind::Bool(actual_val)) => {
        *expected_val == actual_val
      }
      _ => false,
    },
    MemberType::Never => false,
  }
}
