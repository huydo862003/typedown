//! Tracked query for typechecking

use std::any::Any;

use typedown_macros::query_derived;
use typedown_types::diagnostic::Diagnostic;

use crate::derived::get_builtin_types::get_num_type;
use crate::derived::typechecker::get_node_type::get_node_type;
use crate::types::{
  HirValue, HirValueKind, MemberType, TdrDictType, TdrFuncType, TdrListType, TdrProductType,
  TdrTypeLike, TypecheckResult,
};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn typecheck(db: &TypedownDatabase, hir: HirValue) -> TypecheckResult {
  // Synthesize the type of the node
  let type_result = get_node_type(db, hir);
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
      diagnostics.extend(check_mapping_fields(
        db,
        hir,
        &entries,
        declared_type.as_ref(),
      ));
    }
    // Check tag inner matches the tag's schema
    HirValueKind::Tag { inner, .. } => {
      diagnostics.extend(check_tag(db, declared_type.as_ref(), *inner));
    }
    // Check call arity and arg types against function signature
    HirValueKind::Call { callee, args } => {
      diagnostics.extend(check_call(db, *callee, args));
    }
    // Check each item against the list's element type
    HirValueKind::Sequence(items) => {
      diagnostics.extend(check_sequence(db, declared_type.as_ref(), items));
    }
    // Typecheck each embedded expression in an interpolated string
    HirValueKind::Interpolated(parts) => {
      for part in parts {
        if let crate::types::InterpolatedPart::Expr(expr) = part {
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
  expected_type: &dyn TdrTypeLike,
) -> Vec<Diagnostic> {
  let mut diagnostics = vec![];

  for (key, value_hir) in entries {
    if let Some(member) = expected_type.get_field_type(db, key) {
      // Recursively typecheck the field value
      let tc_result = typecheck(db, *value_hir);
      diagnostics.extend(tc_result.diagnostics(db).iter().cloned());

      // Check synthesized type against expected field type
      let value_result = get_node_type(db, *value_hir);
      if let Some(actual_type) = value_result.typ(db) {
        if !member_type_compatible(db, &member.typ(db), actual_type.as_ref()) {
          let node = value_hir.node(db);
          diagnostics.push(Diagnostic::FieldTypeMismatch {
            field: key.clone(),
            expected: member_type_display_name(db, &member.typ(db)),
            start_offset: node.offset(),
            end_offset: node.offset() + node.text_len(),
          });
        }
      }
      // None (any) is always compatible, so no diagnostic.
    }
  }

  // Check required fields are present.
  // Downcast to TdrProductType to enumerate declared fields.
  let mapping_node = mapping_hir.node(db);
  let present_keys: std::collections::HashSet<&str> =
    entries.iter().map(|(key, _)| key.as_str()).collect();

  if let Some(product) = (expected_type as &dyn Any).downcast_ref::<crate::types::TdrProductType>()
  {
    for (field_name, member) in product.fields(db) {
      let is_optional = member
        .descriptors(db)
        .contains(crate::types::TypeMemberDescriptors::OPTIONAL);
      if !is_optional && !present_keys.contains(field_name.as_str()) {
        diagnostics.push(Diagnostic::MissingRequiredField {
          field: field_name.clone(),
          start_offset: mapping_node.offset(),
          end_offset: mapping_node.offset() + mapping_node.text_len(),
        });
      }
    }
  }

  diagnostics
}

fn check_tag(
  db: &TypedownDatabase,
  expected_type: &dyn TdrTypeLike,
  inner: HirValue,
) -> Vec<Diagnostic> {
  let mut diagnostics = vec![];
  let inner_result = get_node_type(db, inner);
  diagnostics.extend(inner_result.diagnostics(db).iter().cloned());
  if let Some(actual_type) = inner_result.typ(db) {
    if !expected_type.is_compatible_with(db, actual_type.as_ref()) {
      let node = inner.node(db);
      diagnostics.push(Diagnostic::TagTypeMismatch {
        expected: expected_type.display_name(db),
        start_offset: node.offset(),
        end_offset: node.offset() + node.text_len(),
      });
    }
  }
  diagnostics
}

fn check_call(db: &TypedownDatabase, callee: HirValue, args: Vec<HirValue>) -> Vec<Diagnostic> {
  let mut diagnostics = vec![];

  let callee_result = get_node_type(db, callee);
  diagnostics.extend(callee_result.diagnostics(db).iter().cloned());

  let callee_type = match callee_result.typ(db) {
    Some(typ) => typ,
    None => return diagnostics,
  };

  let func = match (callee_type.as_ref() as &dyn Any).downcast_ref::<TdrFuncType>() {
    Some(func) => func,
    None => {
      let node = callee.node(db);
      diagnostics.push(Diagnostic::NotCallable {
        start_offset: node.offset(),
        end_offset: node.offset() + node.text_len(),
      });
      return diagnostics;
    }
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
    let arg_result = get_node_type(db, *arg_hir);
    diagnostics.extend(arg_result.diagnostics(db).iter().cloned());
    if let Some(arg_type) = arg_result.typ(db) {
      if !param.is_compatible_with(db, arg_type.as_ref()) {
        let node = arg_hir.node(db);
        diagnostics.push(Diagnostic::ArgTypeMismatch {
          expected: param.display_name(db),
          start_offset: node.offset(),
          end_offset: node.offset() + node.text_len(),
        });
      }
    }
  }

  diagnostics
}

fn check_index(db: &TypedownDatabase, expr: HirValue, indices: Vec<HirValue>) -> Vec<Diagnostic> {
  let mut diagnostics = vec![];

  let expr_result = get_node_type(db, expr);
  diagnostics.extend(expr_result.diagnostics(db).iter().cloned());

  let expr_type = match expr_result.typ(db) {
    Some(typ) => typ,
    None => return diagnostics,
  };

  // Type instantiation - no checking needed, just arity (handled by instantiate_type)
  if expr_type.arity(db) > 0 {
    return diagnostics;
  }

  // List element access: index must be a number
  if (expr_type.as_ref() as &dyn Any)
    .downcast_ref::<TdrListType>()
    .is_some()
  {
    for idx_hir in &indices {
      let idx_result = get_node_type(db, *idx_hir);
      diagnostics.extend(idx_result.diagnostics(db).iter().cloned());
      if let Some(idx_type) = idx_result.typ(db) {
        let num_type = get_num_type(db);
        if !num_type.is_compatible_with(db, idx_type.as_ref()) {
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
  if let Some(dict) = (expr_type.as_ref() as &dyn Any).downcast_ref::<TdrDictType>() {
    if let Some(key_type) = dict.key(db) {
      for idx_hir in &indices {
        let idx_result = get_node_type(db, *idx_hir);
        diagnostics.extend(idx_result.diagnostics(db).iter().cloned());
        if let Some(idx_type) = idx_result.typ(db) {
          if !key_type.is_compatible_with(db, idx_type.as_ref()) {
            let node = idx_hir.node(db);
            diagnostics.push(Diagnostic::IndexTypeMismatch {
              expected: key_type.display_name(db),
              start_offset: node.offset(),
              end_offset: node.offset() + node.text_len(),
            });
          }
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

  let operand_result = get_node_type(db, operand);
  let operand_type = match operand_result.typ(db) {
    Some(typ) => typ,
    None => return diagnostics,
  };

  let expected_type: Box<dyn TdrTypeLike> = match op {
    "-" | "+" => Box::new(get_num_type(db)),
    // ~ is logical not: accepts any type (only null and false are falsy)
    "~" | "!" => return diagnostics,
    _ => return diagnostics,
  };

  if !expected_type.is_compatible_with(db, operand_type.as_ref()) {
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

  let left_type = get_node_type(db, left).typ(db);
  let right_type = get_node_type(db, right).typ(db);

  match op {
    // Arithmetic: both operands must be number
    "+" | "-" | "*" | "/" | "%" | "**" => {
      let num_type = Box::new(get_num_type(db));
      if let Some(lt) = &left_type {
        if !num_type.is_compatible_with(db, lt.as_ref()) {
          let node = left.node(db);
          diagnostics.push(Diagnostic::OperandTypeMismatch {
            op: op.to_string(),
            expected: "number".to_string(),
            start_offset: node.offset(),
            end_offset: node.offset() + node.text_len(),
          });
        }
      }
      if let Some(rt) = &right_type {
        if !num_type.is_compatible_with(db, rt.as_ref()) {
          let node = right.node(db);
          diagnostics.push(Diagnostic::OperandTypeMismatch {
            op: op.to_string(),
            expected: "number".to_string(),
            start_offset: node.offset(),
            end_offset: node.offset() + node.text_len(),
          });
        }
      }
    }
    // Logical: both operands must be boolean
    "&&" | "||" => {
      let bool_type = Box::new(crate::derived::get_builtin_types::get_bool_type(db));
      if let Some(lt) = &left_type {
        if !bool_type.is_compatible_with(db, lt.as_ref()) {
          let node = left.node(db);
          diagnostics.push(Diagnostic::OperandTypeMismatch {
            op: op.to_string(),
            expected: "boolean".to_string(),
            start_offset: node.offset(),
            end_offset: node.offset() + node.text_len(),
          });
        }
      }
      if let Some(rt) = &right_type {
        if !bool_type.is_compatible_with(db, rt.as_ref()) {
          let node = right.node(db);
          diagnostics.push(Diagnostic::OperandTypeMismatch {
            op: op.to_string(),
            expected: "boolean".to_string(),
            start_offset: node.offset(),
            end_offset: node.offset() + node.text_len(),
          });
        }
      }
    }
    // Comparison: no operand type constraint (any types can be compared)
    "==" | "!=" | "<" | ">" | "<=" | ">=" => {}
    _ => {}
  }

  diagnostics
}

fn check_sequence(
  db: &TypedownDatabase,
  declared_type: &dyn TdrTypeLike,
  items: Vec<HirValue>,
) -> Vec<Diagnostic> {
  let mut diagnostics = vec![];

  // Get the element type from the list type
  let elem_type = match (declared_type as &dyn Any).downcast_ref::<TdrListType>() {
    Some(list) => list.elem(db),
    None => return diagnostics,
  };

  let elem_type = match elem_type {
    Some(typ) => typ,
    // Uninstantiated list: no element type constraint
    None => return diagnostics,
  };

  for item in items {
    // Recursively typecheck each item
    let tc_result = typecheck(db, item);
    diagnostics.extend(tc_result.diagnostics(db).iter().cloned());

    // Check item type against element type
    let item_result = get_node_type(db, item);
    if let Some(item_type) = item_result.typ(db) {
      if !elem_type.is_compatible_with(db, item_type.as_ref()) {
        let node = item.node(db);
        diagnostics.push(Diagnostic::FieldTypeMismatch {
          field: "[]".to_string(),
          expected: elem_type.display_name(db),
          start_offset: node.offset(),
          end_offset: node.offset() + node.text_len(),
        });
      }
    }
  }

  diagnostics
}

fn member_type_display_name(db: &TypedownDatabase, member: &MemberType) -> String {
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

fn member_type_compatible(
  db: &TypedownDatabase,
  expected: &MemberType,
  actual: &dyn TdrTypeLike,
) -> bool {
  match expected {
    MemberType::Simple(exp_type) => exp_type.is_compatible_with(db, actual),
    MemberType::Sum(members) => members
      .iter()
      .any(|member| member_type_compatible(db, &member.typ(db), actual)),
    MemberType::Literal(_) => false,
  }
}

#[cfg(test)]
mod tests {
  use typedown_syntax::ast::{AstNode, SourceFile};

  use crate::{
    derived::{hir::lower_expr, parse_file::parse_file, typechecker::typecheck::typecheck},
    fixtures::load_vault_fixture,
  };

  #[test]
  fn typecheck_mapping_without_type_infers_product_no_errors() {
    let (db, project, file) = load_vault_fixture("typecheck/my_vault", "content/literal_value.tdr");
    let root = parse_file(&db, project, file).ast(&db);
    let mapping = SourceFile::cast(root)
      .unwrap()
      .frontmatter()
      .unwrap()
      .mapping()
      .unwrap();
    let hir = lower_expr(&db, project, file, mapping.syntax().clone());

    let result = typecheck(&db, hir);
    assert!(
      result.diagnostics(&db).is_empty(),
      "mapping without _type infers product type, no errors expected: {:?}",
      result.diagnostics(&db)
    );
  }

  #[test]
  fn typecheck_unresolved_type_has_diagnostics() {
    let (db, project, file) =
      load_vault_fixture("typecheck/my_vault", "content/unresolved_type.tdr");
    let root = parse_file(&db, project, file).ast(&db);
    let mapping = SourceFile::cast(root)
      .unwrap()
      .frontmatter()
      .unwrap()
      .mapping()
      .unwrap();
    let hir = lower_expr(&db, project, file, mapping.syntax().clone());

    let result = typecheck(&db, hir);
    assert!(
      !result.diagnostics(&db).is_empty(),
      "expected diagnostics for unresolved schema"
    );
  }

  #[test]
  fn typecheck_mapping_with_ident_value() {
    let (db, project, file) = load_vault_fixture("typecheck/my_vault", "content/ident_value.tdr");
    let root = parse_file(&db, project, file).ast(&db);
    let mapping = SourceFile::cast(root)
      .unwrap()
      .frontmatter()
      .unwrap()
      .mapping()
      .unwrap();
    let hir = lower_expr(&db, project, file, mapping.syntax().clone());

    let result = typecheck(&db, hir);
    assert!(
      result.diagnostics(&db).is_empty(),
      "expected no diagnostics, got: {:?}",
      result.diagnostics(&db)
    );
  }

  #[test]
  fn typecheck_schema_missing_required_field_has_diagnostics() {
    let (db, project, file) = load_vault_fixture(
      "typecheck/my_vault",
      "content/schema_missing_properties.tdr",
    );
    let root = parse_file(&db, project, file).ast(&db);
    let mapping = SourceFile::cast(root)
      .unwrap()
      .frontmatter()
      .unwrap()
      .mapping()
      .unwrap();
    let hir = lower_expr(&db, project, file, mapping.syntax().clone());

    let result = typecheck(&db, hir);
    let diags = result.diagnostics(&db);
    assert!(
      diags.iter().any(|d| matches!(d, typedown_types::diagnostic::Diagnostic::MissingRequiredField { field, .. } if field == "properties")),
      "expected MissingRequiredField for 'properties', got: {:?}",
      diags
    );
  }
}
