//! Tracked query for typechecking

use std::collections::HashSet;

use crate::syntax::diagnostic::Diagnostic;
use typedown_macros::query_derived;

use crate::db::TypedownDatabase;
use crate::db::derived::get_builtin_types::{get_bool_type, get_num_type};
use crate::db::derived::name_resolver::referee::referee;
use crate::db::derived::typechecker::infer_node_type::infer_node_type;
use crate::db::types::{
  HirValue, HirValueKind, InterpolatedPart, LiteralValue, MemberType, TdrTypeEnum, TdrTypeLike,
  TypeMember, TypeMemberDescriptors, TypecheckResult, member_type_display_name,
};
use typedown_incremental::QueryDatabase;

#[query_derived]
pub fn typecheck(db: &TypedownDatabase, hir: HirValue) -> TypecheckResult {
  // Synthesize the type of the node
  let type_result = infer_node_type(db, hir);
  let mut diagnostics = type_result.diagnostics(db).clone();

  // If type is None (any), nothing to check
  let declared_type = match type_result.typ(db) {
    Some(typ) => typ,
    None => return TypecheckResult::new(db, diagnostics),
  };

  // Validate structure based on the node kind
  match hir.kind(db) {
    // Check mapping fields against declared schema type
    HirValueKind::Mapping(entries) => {
      diagnostics.extend(check_mapping_fields(db, hir, &entries, &declared_type));
    }
    // Check tag inner matches the tag's schema
    HirValueKind::Tag { inner, .. } => {
      diagnostics.extend(check_tag(db, &declared_type, *inner));
    }
    // Check call arity and arg types against function signature
    HirValueKind::Call { callee, args } => {
      diagnostics.extend(check_call(db, *callee, args));
    }
    // Check each item against the list's element type
    HirValueKind::Sequence(items) => {
      diagnostics.extend(check_sequence(db, &declared_type, items));
    }
    // Typecheck each embedded expression in an interpolated string
    HirValueKind::Interpolated(parts) | HirValueKind::Markdown(parts) => {
      for part in parts {
        if let InterpolatedPart::Expr(expr) = part {
          let tc_result = typecheck(db, expr);
          diagnostics.extend(tc_result.diagnostics(db).iter().cloned());
        }
      }
    }
    // Check unary operand type
    HirValueKind::Unary { op, operand } => {
      diagnostics.extend(check_unary(db, &op, *operand));
    }
    // Check binary operand types
    HirValueKind::Binary { op, left, right } => {
      diagnostics.extend(check_binary(db, &op, *left, *right));
    }
    // Check index types against container key types
    HirValueKind::Index { expr, indices } => {
      diagnostics.extend(check_index(db, *expr, indices));
    }
    _ => {}
  }

  TypecheckResult::new(db, diagnostics)
}

fn check_mapping_fields(
  db: &TypedownDatabase,
  mapping_hir: HirValue,
  entries: &[(String, HirValue)],
  expected_type: &TdrTypeEnum,
) -> Vec<Diagnostic> {
  let mut diagnostics = vec![];

  for (key, value_hir) in entries {
    // _type requires the value to resolve to a schema symbol
    if key == "_type" {
      let resolved = referee(db, *value_hir);
      if let Some(symbol) = resolved.value(db)
        && !symbol.kind(db).is_schema()
      {
        let node = value_hir.node(db);
        diagnostics.push(Diagnostic::FieldTypeMismatch {
          field: "_type".to_string(),
          expected: "schema".to_string(),
          start_offset: node.offset(),
          end_offset: node.offset() + node.text_len(),
        });
      }
      continue;
    }
    if let Some(member) = expected_type.get_field_type(db, key) {
      // Recursively typecheck the field value
      let tc_result = typecheck(db, *value_hir);
      diagnostics.extend(tc_result.diagnostics(db).iter().cloned());

      // Check synthesized type against expected field type
      let value_result = infer_node_type(db, *value_hir);
      let is_optional = member
        .descriptors(db)
        .contains(TypeMemberDescriptors::OPTIONAL);
      match value_result.typ(db) {
        Some(actual_type) => {
          let matches = value_matches_member_type(db, &member.typ(db), &actual_type, *value_hir);
          if !matches {
            let node = value_hir.node(db);
            diagnostics.push(Diagnostic::FieldTypeMismatch {
              field: key.clone(),
              expected: member_type_display_name(db, &member.typ(db)),
              start_offset: node.offset(),
              end_offset: node.offset() + node.text_len(),
            });
          }
        }
        // Null on a non-optional field is a type error
        None if !is_optional => {
          let node = value_hir.node(db);
          diagnostics.push(Diagnostic::FieldTypeMismatch {
            field: key.clone(),
            expected: member_type_display_name(db, &member.typ(db)),
            start_offset: node.offset(),
            end_offset: node.offset() + node.text_len(),
          });
        }
        None => {}
      }
    }
  }

  // Check required fields are present (not null are checked above)
  let mapping_node = mapping_hir.node(db);
  let present_keys: HashSet<&str> = entries.iter().map(|(key, _)| key.as_str()).collect();

  // Enumerate declared fields to check required ones are present
  let declared_fields: Vec<(String, TypeMember)> =
    if let TdrTypeEnum::TdrProductType(product) = &expected_type {
      product.fields(db).into_iter().collect()
    } else if expected_type.is_tdr_schema_type() {
      // TdrSchemaType has a fixed set of fields
      vec!["properties"]
        .into_iter()
        .filter_map(|name| {
          expected_type
            .get_owned_field_type(db, name)
            .map(|member| (name.to_string(), member))
        })
        .collect()
    } else {
      vec![]
    };

  for (field_name, member) in declared_fields {
    let is_optional = member
      .descriptors(db)
      .contains(TypeMemberDescriptors::OPTIONAL);
    if !is_optional && !present_keys.contains(field_name.as_str()) {
      diagnostics.push(Diagnostic::MissingRequiredField {
        field: field_name,
        start_offset: mapping_node.offset(),
        end_offset: mapping_node.offset() + mapping_node.text_len(),
      });
    }
  }

  diagnostics
}

fn check_tag(
  db: &TypedownDatabase,
  expected_type: &TdrTypeEnum,
  inner: HirValue,
) -> Vec<Diagnostic> {
  let mut diagnostics = vec![];
  let inner_result = infer_node_type(db, inner);
  diagnostics.extend(inner_result.diagnostics(db).iter().cloned());
  if let Some(actual_type) = inner_result.typ(db)
    && !expected_type.is_compatible_with(db, &actual_type)
  {
    let node = inner.node(db);
    diagnostics.push(Diagnostic::TagTypeMismatch {
      expected: expected_type.display_name(db),
      start_offset: node.offset(),
      end_offset: node.offset() + node.text_len(),
    });
  }
  diagnostics
}

fn check_call(db: &TypedownDatabase, callee: HirValue, args: Vec<HirValue>) -> Vec<Diagnostic> {
  let mut diagnostics = vec![];

  let callee_result = infer_node_type(db, callee);
  diagnostics.extend(callee_result.diagnostics(db).iter().cloned());

  let callee_type = match callee_result.typ(db) {
    Some(typ) => typ,
    None => return diagnostics,
  };

  let Some(func) = callee_type.as_tdr_func_type() else {
    let node = callee.node(db);
    diagnostics.push(Diagnostic::NotCallable {
      start_offset: node.offset(),
      end_offset: node.offset() + node.text_len(),
    });
    return diagnostics;
  };

  let sig = func.signature(db);
  let params = sig.params(db);

  if params.len() != args.len() {
    let node = callee.node(db);
    diagnostics.push(Diagnostic::WrongArgCount {
      expected: params.len(),
      got: args.len(),
      start_offset: node.offset(),
      end_offset: node.offset() + node.text_len(),
    });
    return diagnostics;
  }

  for (param, arg_hir) in params.iter().zip(args.iter()) {
    let arg_result = infer_node_type(db, *arg_hir);
    diagnostics.extend(arg_result.diagnostics(db).iter().cloned());
    if let Some(arg_type) = arg_result.typ(db)
      && !param.is_compatible_with(db, &arg_type)
    {
      let node = arg_hir.node(db);
      diagnostics.push(Diagnostic::ArgTypeMismatch {
        expected: param.display_name(db),
        start_offset: node.offset(),
        end_offset: node.offset() + node.text_len(),
      });
    }
  }

  diagnostics
}

fn check_index(db: &TypedownDatabase, expr: HirValue, indices: Vec<HirValue>) -> Vec<Diagnostic> {
  let mut diagnostics = vec![];

  let expr_result = infer_node_type(db, expr);
  diagnostics.extend(expr_result.diagnostics(db).iter().cloned());

  let expr_type = match expr_result.typ(db) {
    Some(typ) => typ,
    None => return diagnostics,
  };

  // Type instantiation: no checking is needed because we do not support type bound, only check arity
  if expr_type.arity(db) > 0 {
    return diagnostics;
  }

  // List element access: index must be a number
  if expr_type.is_tdr_list_type() {
    for idx_hir in &indices {
      let idx_result = infer_node_type(db, *idx_hir);
      diagnostics.extend(idx_result.diagnostics(db).iter().cloned());
      if let Some(idx_type) = idx_result.typ(db) {
        let num_type = get_num_type(db);
        if !num_type.is_compatible_with(db, &idx_type) {
          let node = idx_hir.node(db);
          diagnostics.push(Diagnostic::IndexTypeMismatch {
            expected: "number".to_string(),
            start_offset: node.offset(),
            end_offset: node.offset() + node.text_len(),
          });
        }
      }
    }
    return diagnostics;
  }

  // Dict element access: index must match key type
  if let TdrTypeEnum::TdrDictType(dict) = &expr_type {
    if let Some(key_type) = dict.key(db) {
      for idx_hir in &indices {
        let idx_result = infer_node_type(db, *idx_hir);
        diagnostics.extend(idx_result.diagnostics(db).iter().cloned());
        if let Some(idx_type) = idx_result.typ(db)
          && !key_type.is_compatible_with(db, &idx_type)
        {
          let node = idx_hir.node(db);
          diagnostics.push(Diagnostic::IndexTypeMismatch {
            expected: key_type.display_name(db),
            start_offset: node.offset(),
            end_offset: node.offset() + node.text_len(),
          });
        }
      }
    }
    return diagnostics;
  }

  // String indexing is valid: index must be a number
  if expr_type.is_tdr_str_type() {
    for idx_hir in &indices {
      let idx_result = infer_node_type(db, *idx_hir);
      diagnostics.extend(idx_result.diagnostics(db).iter().cloned());
      if let Some(idx_type) = idx_result.typ(db) {
        let num_type = get_num_type(db);
        if !num_type.is_compatible_with(db, &idx_type) {
          let node = idx_hir.node(db);
          diagnostics.push(Diagnostic::IndexTypeMismatch {
            expected: "number".to_string(),
            start_offset: node.offset(),
            end_offset: node.offset() + node.text_len(),
          });
        }
      }
    }
    return diagnostics;
  }

  // Not indexable
  let node = expr.node(db);
  diagnostics.push(Diagnostic::NotIndexable {
    start_offset: node.offset(),
    end_offset: node.offset() + node.text_len(),
  });

  diagnostics
}

fn check_unary(db: &TypedownDatabase, op: &str, operand: HirValue) -> Vec<Diagnostic> {
  let mut diagnostics = vec![];

  let tc_result = typecheck(db, operand);
  diagnostics.extend(tc_result.diagnostics(db).iter().cloned());

  let operand_result = infer_node_type(db, operand);
  let operand_type = match operand_result.typ(db) {
    Some(typ) => typ,
    None => return diagnostics,
  };

  let expected_type: TdrTypeEnum = match op {
    "-" | "+" => get_num_type(db).into(),
    // ~ is logical not: accepts any type (only null and false are falsy)
    "~" => return diagnostics,
    _ => return diagnostics,
  };

  if !expected_type.is_compatible_with(db, &operand_type) {
    let node = operand.node(db);
    diagnostics.push(Diagnostic::OperandTypeMismatch {
      op: op.to_string(),
      expected: expected_type.display_name(db),
      start_offset: node.offset(),
      end_offset: node.offset() + node.text_len(),
    });
  }

  diagnostics
}

fn check_binary(
  db: &TypedownDatabase,
  op: &str,
  left: HirValue,
  right: HirValue,
) -> Vec<Diagnostic> {
  let mut diagnostics = vec![];

  let tc_left = typecheck(db, left);
  diagnostics.extend(tc_left.diagnostics(db).iter().cloned());
  let tc_right = typecheck(db, right);
  diagnostics.extend(tc_right.diagnostics(db).iter().cloned());

  let left_type = infer_node_type(db, left).typ(db);
  let right_type = infer_node_type(db, right).typ(db);

  match op {
    // Arithmetic: both operands must be number
    "+" | "-" | "*" | "/" | "%" | "**" => {
      let num_type: TdrTypeEnum = get_num_type(db).into();
      if let Some(lt) = &left_type
        && !num_type.is_compatible_with(db, lt)
      {
        let node = left.node(db);
        diagnostics.push(Diagnostic::OperandTypeMismatch {
          op: op.to_string(),
          expected: "number".to_string(),
          start_offset: node.offset(),
          end_offset: node.offset() + node.text_len(),
        });
      }
      if let Some(rt) = &right_type
        && !num_type.is_compatible_with(db, rt)
      {
        let node = right.node(db);
        diagnostics.push(Diagnostic::OperandTypeMismatch {
          op: op.to_string(),
          expected: "number".to_string(),
          start_offset: node.offset(),
          end_offset: node.offset() + node.text_len(),
        });
      }
    }
    // Logical: both operands must be boolean
    // Consider allow truthy and falsy?
    "&&" | "||" => {
      let bool_type: TdrTypeEnum = get_bool_type(db).into();
      if let Some(lt) = &left_type
        && !bool_type.is_compatible_with(db, lt)
      {
        let node = left.node(db);
        diagnostics.push(Diagnostic::OperandTypeMismatch {
          op: op.to_string(),
          expected: "boolean".to_string(),
          start_offset: node.offset(),
          end_offset: node.offset() + node.text_len(),
        });
      }
      if let Some(rt) = &right_type
        && !bool_type.is_compatible_with(db, rt)
      {
        let node = right.node(db);
        diagnostics.push(Diagnostic::OperandTypeMismatch {
          op: op.to_string(),
          expected: "boolean".to_string(),
          start_offset: node.offset(),
          end_offset: node.offset() + node.text_len(),
        });
      }
    }
    // Comparison: any type can be compared
    // :)) not sure
    "==" | "!=" | "<" | ">" | "<=" | ">=" => {}
    _ => {}
  }

  diagnostics
}

fn check_sequence(
  db: &TypedownDatabase,
  declared_type: &TdrTypeEnum,
  items: Vec<HirValue>,
) -> Vec<Diagnostic> {
  let mut diagnostics = vec![];

  // Get the element type from the list type
  let Some(list) = declared_type.as_tdr_list_type() else {
    return diagnostics;
  };

  let elem_type = match list.elem(db) {
    Some(typ) => typ,
    // Uninstantiated list: no element type constraint
    None => return diagnostics,
  };

  for item in items {
    // Recursively typecheck each item
    let tc_result = typecheck(db, item);
    diagnostics.extend(tc_result.diagnostics(db).iter().cloned());

    // Check item type against element type
    let item_result = infer_node_type(db, item);
    if let Some(item_type) = item_result.typ(db)
      && !elem_type.is_compatible_with(db, &item_type)
    {
      let node = item.node(db);
      diagnostics.push(Diagnostic::ElementTypeMismatch {
        expected: elem_type.display_name(db),
        start_offset: node.offset(),
        end_offset: node.offset() + node.text_len(),
      });
    }
  }

  diagnostics
}

fn value_matches_member_type(
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

#[cfg(test)]
mod tests {
  use crate::db::{
    derived::typechecker::typecheck::typecheck, fixtures::load_vault_fixture, utils::lower_file,
  };
  use crate::syntax::diagnostic::Diagnostic;

  // Mapping without _type: infers product type, no validation errors
  #[test]
  fn typecheck_mapping_without_type_infers_product_no_errors() {
    let (db, project, file) = load_vault_fixture("typecheck/my_vault", "content/literal_value.tdr");
    let (hir, _) = lower_file(&db, project, file);
    let result = typecheck(&db, hir.unwrap());
    assert!(
      result.diagnostics(&db).is_empty(),
      "mapping without _type infers product type, no errors expected: {:?}",
      result.diagnostics(&db)
    );
  }

  // _type references a non-existent schema
  #[test]
  fn typecheck_unresolved_type_has_diagnostics() {
    let (db, project, file) =
      load_vault_fixture("typecheck/my_vault", "content/unresolved_type.tdr");
    let (hir, _) = lower_file(&db, project, file);
    let result = typecheck(&db, hir.unwrap());
    assert!(
      !result.diagnostics(&db).is_empty(),
      "expected diagnostics for unresolved schema"
    );
  }

  // Mapping with identifier value that resolves to nothing: no errors (any type)
  #[test]
  fn typecheck_mapping_with_ident_value() {
    let (db, project, file) = load_vault_fixture("typecheck/my_vault", "content/ident_value.tdr");
    let (hir, _) = lower_file(&db, project, file);
    let result = typecheck(&db, hir.unwrap());
    assert!(
      result.diagnostics(&db).is_empty(),
      "expected no diagnostics, got: {:?}",
      result.diagnostics(&db)
    );
  }

  // Schema with _type: Schema but missing required 'properties' field
  #[test]
  fn typecheck_schema_missing_properties_has_diagnostics() {
    let (db, project, file) = load_vault_fixture(
      "typecheck/my_vault",
      "content/schema_missing_properties.tdr",
    );
    let (hir, _) = lower_file(&db, project, file);
    let result = typecheck(&db, hir.unwrap());
    let diags = result.diagnostics(&db);
    assert!(
      diags.iter().any(
        |d| matches!(d, Diagnostic::MissingRequiredField { field, .. } if field == "properties")
      ),
      "expected MissingRequiredField for 'properties', got: {:?}",
      diags
    );
  }

  // Typecheck a valid document against a user-defined schema
  #[test]
  fn typecheck_valid_person_no_errors() {
    let (db, project, file) = load_vault_fixture("typecheck/my_vault", "content/valid_person.tdr");
    let (hir, _) = lower_file(&db, project, file);
    let result = typecheck(&db, hir.unwrap());
    assert!(
      result.diagnostics(&db).is_empty(),
      "valid Person should have no errors: {:?}",
      result.diagnostics(&db)
    );
  }

  // Field type mismatch: name expects string, got number
  #[test]
  fn typecheck_wrong_field_type_has_diagnostics() {
    let (db, project, file) =
      load_vault_fixture("typecheck/my_vault", "content/wrong_field_type.tdr");
    let (hir, _) = lower_file(&db, project, file);
    let result = typecheck(&db, hir.unwrap());
    let diags = result.diagnostics(&db);
    assert!(
      diags.iter().any(|d| matches!(d, Diagnostic::FieldTypeMismatch { field, expected, .. } if field == "name" && expected == "string")),
      "expected FieldTypeMismatch for 'name' with expected 'string', got: {:?}",
      diags
    );
  }

  // Recursive typecheck for nested inline object with valid types
  #[test]
  fn typecheck_nested_valid_no_errors() {
    let (db, project, file) = load_vault_fixture("typecheck/my_vault", "content/nested_valid.tdr");
    let (hir, _) = lower_file(&db, project, file);
    let result = typecheck(&db, hir.unwrap());
    assert!(
      result.diagnostics(&db).is_empty(),
      "valid nested PersonWithAddress should have no errors: {:?}",
      result.diagnostics(&db)
    );
  }

  // Recursive typecheck for nested inline object with wrong field type (street: 42 instead of string)
  #[test]
  fn typecheck_nested_wrong_type_has_diagnostics() {
    let (db, project, file) =
      load_vault_fixture("typecheck/my_vault", "content/nested_wrong_type.tdr");
    let (hir, _) = lower_file(&db, project, file);
    let result = typecheck(&db, hir.unwrap());
    let diags = result.diagnostics(&db);
    assert!(
      diags
        .iter()
        .any(|d| matches!(d, Diagnostic::FieldTypeMismatch { field, .. } if field == "address")),
      "expected FieldTypeMismatch for 'address', got: {:?}",
      diags
    );
  }

  // Unary minus on number: no errors
  #[test]
  fn typecheck_unary_valid() {
    let (db, project, file) = load_vault_fixture("typecheck/my_vault", "content/unary_valid.tdr");
    let (hir, _) = lower_file(&db, project, file);
    let result = typecheck(&db, hir.unwrap());
    assert!(
      result.diagnostics(&db).is_empty(),
      "unary minus on number should have no errors: {:?}",
      result.diagnostics(&db)
    );
  }

  // Unary minus on boolean: OperandTypeMismatch
  #[test]
  fn typecheck_unary_wrong_type() {
    let (db, project, file) =
      load_vault_fixture("typecheck/my_vault", "content/unary_wrong_type.tdr");
    let (hir, _) = lower_file(&db, project, file);
    let result = typecheck(&db, hir.unwrap());
    let diags = result.diagnostics(&db);
    assert!(
      diags
        .iter()
        .any(|d| matches!(d, Diagnostic::OperandTypeMismatch { .. })),
      "expected OperandTypeMismatch for unary minus on boolean, got: {:?}",
      diags
    );
  }

  // Binary addition of numbers: no errors
  #[test]
  fn typecheck_binary_valid() {
    let (db, project, file) = load_vault_fixture("typecheck/my_vault", "content/binary_valid.tdr");
    let (hir, _) = lower_file(&db, project, file);
    let result = typecheck(&db, hir.unwrap());
    assert!(
      result.diagnostics(&db).is_empty(),
      "binary addition of numbers should have no errors: {:?}",
      result.diagnostics(&db)
    );
  }

  // Binary addition with boolean operand: OperandTypeMismatch
  #[test]
  fn typecheck_binary_wrong_type() {
    let (db, project, file) =
      load_vault_fixture("typecheck/my_vault", "content/binary_wrong_type.tdr");
    let (hir, _) = lower_file(&db, project, file);
    let result = typecheck(&db, hir.unwrap());
    let diags = result.diagnostics(&db);
    assert!(
      diags
        .iter()
        .any(|d| matches!(d, Diagnostic::OperandTypeMismatch { .. })),
      "expected OperandTypeMismatch for binary addition with boolean, got: {:?}",
      diags
    );
  }

  #[test]
  fn typecheck_math_field_valid() {
    let (db, project, file) = load_vault_fixture("typecheck/my_vault", "content/valid_math.tdr");
    let (hir, _) = lower_file(&db, project, file);
    let result = typecheck(&db, hir.unwrap());
    assert!(
      result.diagnostics(&db).is_empty(),
      "math field should typecheck with no errors: {:?}",
      result.diagnostics(&db)
    );
  }

  #[test]
  fn typecheck_markdown_body_with_interpolation() {
    let (db, project, file) =
      load_vault_fixture("typecheck/my_vault", "content/valid_markdown.tdr");
    let (hir, _) = lower_file(&db, project, file);
    let result = typecheck(&db, hir.unwrap());
    assert!(
      result.diagnostics(&db).is_empty(),
      "markdown body with interpolation should typecheck with no errors: {:?}",
      result.diagnostics(&db)
    );
  }

  // Quoted string values for date/time/datetime fields should pass typechecking
  // because infer_node_type deduces the specific subtype from ISO format
  #[test]
  fn typecheck_literal_type_valid() {
    let (db, project, file) = load_vault_fixture("typecheck/my_vault", "content/valid_status.tdr");
    let (hir, _) = lower_file(&db, project, file);
    let result = typecheck(&db, hir.unwrap());
    assert!(
      result.diagnostics(&db).is_empty(),
      "state: \"draft\" should match literal type \"draft\": {:?}",
      result.diagnostics(&db)
    );
  }

  #[test]
  fn typecheck_literal_type_mismatch() {
    let (db, project, file) =
      load_vault_fixture("typecheck/my_vault", "content/invalid_status.tdr");
    let (hir, _) = lower_file(&db, project, file);
    let result = typecheck(&db, hir.unwrap());
    let diags = result.diagnostics(&db);
    assert!(
      diags
        .iter()
        .any(|d| matches!(d, Diagnostic::FieldTypeMismatch { field, .. } if field == "state")),
      "state: \"published\" should fail literal type \"draft\": {:?}",
      diags
    );
  }

  #[test]
  fn typecheck_date_time_fields_accept_quoted_strings() {
    let (db, project, file) = load_vault_fixture("typecheck/my_vault", "content/valid_event.tdr");
    let (hir, _) = lower_file(&db, project, file);
    let result = typecheck(&db, hir.unwrap());
    assert!(
      result.diagnostics(&db).is_empty(),
      "date/time/datetime fields should accept quoted string values: {:?}",
      result.diagnostics(&db)
    );
  }
}
