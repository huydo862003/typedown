//! Tracked query for typechecking

use std::any::Any;

use typedown_macros::query_derived;
use typedown_types::diagnostic::Diagnostic;

use crate::derived::get_builtin_types::get_num_type;
use crate::derived::typechecker::get_node_type::get_node_type;
use crate::types::{
  HirValue, HirValueKind, MemberType, TdrDictType, TdrFuncType, TdrListType, TdrTypeLike,
  TypecheckResult,
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
      let value_result = get_node_type(db, *value_hir);
      diagnostics.extend(value_result.diagnostics(db).iter().cloned());
      if let Some(actual_type) = value_result.typ(db) {
        if !member_type_compatible(db, &member.typ(db), actual_type.as_ref()) {
          let node = value_hir.node(db);
          diagnostics.push(Diagnostic::FieldTypeMismatch {
            field: key.clone(),
            expected: String::new(),
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
        expected: String::new(),
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
          expected: String::new(),
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
              expected: String::new(),
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
