//! Evaluate a resource file into typed objects

use typedown_macros::query_derived;

use crate::derived::evaluate::evaluate_node::evaluate_node;
use crate::types::{ResourceResult, Symbol, SymbolKind};
use crate::utils::lower_frontmatter;
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn evaluate_resource(db: &TypedownDatabase, symbol: Symbol) -> ResourceResult {
  let (project, file) = match symbol.kind(db) {
    SymbolKind::UserDefinedResource(project, file) => (project, file),
    _ => return ResourceResult::new(db, None, vec![]),
  };

  let (hir, mut diagnostics) = lower_frontmatter(db, project, file);
  let hir = match hir {
    Some(hir) => hir,
    None => return ResourceResult::new(db, None, diagnostics),
  };

  let node_result = evaluate_node(db, hir);
  diagnostics.extend(node_result.diagnostics(db).iter().cloned());

  ResourceResult::new(db, node_result.value(db), diagnostics)
}

#[cfg(test)]
mod tests {
  use std::any::Any;

  use crate::{
    derived::evaluate::evaluate_node::evaluate_node,
    derived::evaluate::evaluate_resource::evaluate_resource,
    derived::name_resolver::file_symbol::file_symbol,
    fixtures::load_vault_fixture,
    types::{HirValueKind, TdrBoolObj, TdrNumObj, TdrProductType, TdrStrObj},
    utils::lower_frontmatter,
  };

  // A valid resource with _type produces an object with the declared fields
  #[test]
  fn evaluate_resource_valid_person() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "content/valid_person.tdr");
    let symbol = file_symbol(&db, project, file)
      .value(&db)
      .expect("file_symbol should return a resource symbol");

    let result = evaluate_resource(&db, symbol);
    assert!(
      result.value(&db).is_some(),
      "should produce an object, diagnostics: {:?}",
      result.diagnostics(&db)
    );
    let obj = result.value(&db).unwrap();
    let name_obj = obj.get_owned_field(&db, "name").expect("should have name");
    let name_str = (name_obj.as_ref() as &dyn Any)
      .downcast_ref::<TdrStrObj>()
      .expect("name should be TdrStrObj");
    assert_eq!(name_str.value(&db), "Alice");
  }

  // A field value that doesn't match the declared schema type produces diagnostics
  #[test]
  fn evaluate_resource_wrong_type_has_diagnostics() {
    let (db, project, file) =
      load_vault_fixture("evaluate/my_vault", "content/wrong_field_type.tdr");
    let symbol = file_symbol(&db, project, file)
      .value(&db)
      .expect("file_symbol should return a resource symbol");

    let result = evaluate_resource(&db, symbol);
    assert!(
      !result.diagnostics(&db).is_empty(),
      "should have diagnostics for wrong field type"
    );
  }

  // A schema file placed in content dir is treated as a resource, not a schema
  // but evaluate_resource still produces a value (a TdrProductType)
  #[test]
  fn schema_in_content_dir_is_resource() {
    let (db, project, file) =
      load_vault_fixture("evaluate/my_vault", "content/schema_in_content.tdr");
    let symbol = file_symbol(&db, project, file)
      .value(&db)
      .expect("file_symbol should return a symbol");

    assert!(
      symbol.kind(&db).is_resource(),
      "schema file in content dir should be a resource symbol"
    );

    let result = evaluate_resource(&db, symbol);
    assert!(
      result.value(&db).is_some(),
      "should produce an object, diagnostics: {:?}",
      result.diagnostics(&db)
    );
    let obj = result.value(&db).unwrap();
    let product_type = (obj.as_ref() as &dyn Any)
      .downcast_ref::<TdrProductType>()
      .expect("schema in content dir should produce TdrProductType");
    assert!(
      product_type.fields(&db).contains_key("title"),
      "should have title field"
    );
  }

  // Circular fref does not cause infinite recursion due to lazy evaluation
  #[test]
  fn circular_fref_does_not_panic() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "content/circular_a.tdr");
    let symbol = file_symbol(&db, project, file)
      .value(&db)
      .expect("should return a resource symbol");

    let result = evaluate_resource(&db, symbol);
    assert!(
      result.value(&db).is_some(),
      "circular fref should still produce an object"
    );

    // Access a non-fref field to verify the object works
    let obj = result.value(&db).unwrap();
    let name_obj = obj.get_owned_field(&db, "name").expect("should have name");
    let name_str = (name_obj.as_ref() as &dyn Any)
      .downcast_ref::<TdrStrObj>()
      .expect("name should be TdrStrObj");
    assert_eq!(name_str.value(&db), "Alice");
  }

  // Lazy field access: accessing the fref field evaluates the target on both sides
  #[test]
  fn lazy_fref_field_access() {
    let (db, project, file_a) = load_vault_fixture("evaluate/my_vault", "content/circular_a.tdr");
    let symbol_a = file_symbol(&db, project, file_a)
      .value(&db)
      .expect("should return a resource symbol");

    let result_a = evaluate_resource(&db, symbol_a);
    let alice = result_a.value(&db).unwrap();

    // Alice -> friend -> Bob
    let friend = alice
      .get_owned_field(&db, "friend")
      .expect("should have friend");
    let friend_name = friend
      .get_owned_field(&db, "name")
      .expect("friend should have name");
    let friend_name_str = (friend_name.as_ref() as &dyn Any)
      .downcast_ref::<TdrStrObj>()
      .expect("friend name should be TdrStrObj");
    assert_eq!(friend_name_str.value(&db), "Bob");

    // Bob -> friend -> Alice (circular, should not panic)
    let friend_of_friend = friend
      .get_owned_field(&db, "friend")
      .expect("Bob should have friend");
    let fof_name = friend_of_friend
      .get_owned_field(&db, "name")
      .expect("should have name");
    let fof_name_str = (fof_name.as_ref() as &dyn Any)
      .downcast_ref::<TdrStrObj>()
      .expect("should be TdrStrObj");
    assert_eq!(fof_name_str.value(&db), "Alice");
  }

  // str.to_string() returns the same string value
  #[test]
  fn str_to_string_produces_same_value() {
    let (db, project, file) =
      load_vault_fixture("evaluate/my_vault", "content/str_method_call.tdr");
    let symbol = file_symbol(&db, project, file)
      .value(&db)
      .expect("file_symbol should return a resource symbol");
    let result = evaluate_resource(&db, symbol);
    let obj = result.value(&db).expect("should produce an object");
    let result_field = obj
      .get_owned_field(&db, "result")
      .expect("should have result field");
    let str_obj = (result_field.as_ref() as &dyn Any)
      .downcast_ref::<TdrStrObj>()
      .expect("result should be TdrStrObj");
    assert_eq!(str_obj.value(&db), "hello");
  }

  // num.to_string() returns the decimal representation, without trailing .0 for integers
  #[test]
  fn num_to_string_produces_string_repr() {
    let (db, project, file) =
      load_vault_fixture("evaluate/my_vault", "content/num_method_call.tdr");
    let symbol = file_symbol(&db, project, file)
      .value(&db)
      .expect("file_symbol should return a resource symbol");
    let result = evaluate_resource(&db, symbol);
    let obj = result.value(&db).expect("should produce an object");
    let result_field = obj
      .get_owned_field(&db, "result")
      .expect("should have result field");
    let str_obj = (result_field.as_ref() as &dyn Any)
      .downcast_ref::<TdrStrObj>()
      .expect("result should be TdrStrObj");
    assert_eq!(str_obj.value(&db), "42");
  }

  // bool.to_string() returns "true" or "false"
  #[test]
  fn bool_to_string_produces_string_repr() {
    let (db, project, file) =
      load_vault_fixture("evaluate/my_vault", "content/bool_method_call.tdr");
    let symbol = file_symbol(&db, project, file)
      .value(&db)
      .expect("file_symbol should return a resource symbol");
    let result = evaluate_resource(&db, symbol);
    let obj = result.value(&db).expect("should produce an object");
    let result_field = obj
      .get_owned_field(&db, "result")
      .expect("should have result field");
    let str_obj = (result_field.as_ref() as &dyn Any)
      .downcast_ref::<TdrStrObj>()
      .expect("result should be TdrStrObj");
    assert_eq!(str_obj.value(&db), "true");
  }

  // fref("file.tdr").prop evaluates the referenced resource and accesses a field on it
  #[test]
  fn fref_prop_accesses_field_on_referenced_resource() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "content/fref_prop.tdr");
    let symbol = file_symbol(&db, project, file)
      .value(&db)
      .expect("file_symbol should return a resource symbol");
    let result = evaluate_resource(&db, symbol);
    let obj = result.value(&db).expect("should produce an object");
    let result_field = obj
      .get_owned_field(&db, "result")
      .expect("should have result field");
    let str_obj = (result_field.as_ref() as &dyn Any)
      .downcast_ref::<TdrStrObj>()
      .expect("result should be TdrStrObj");
    assert_eq!(str_obj.value(&db), "Alice");
  }

  // self.field accesses a field on the current resource object
  #[test]
  fn self_ref_accesses_own_field() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "content/self_ref.tdr");
    let symbol = file_symbol(&db, project, file)
      .value(&db)
      .expect("file_symbol should return a resource symbol");
    let result = evaluate_resource(&db, symbol);
    let obj = result.value(&db).expect("should produce an object");
    let result_field = obj
      .get_owned_field(&db, "result")
      .expect("should have result field");
    let str_obj = (result_field.as_ref() as &dyn Any)
      .downcast_ref::<TdrStrObj>()
      .expect("result should be TdrStrObj");
    assert_eq!(str_obj.value(&db), "Alice");
  }

  // String interpolation evaluates embedded expressions and concatenates the parts
  #[test]
  fn str_interp_evaluates_expr_parts() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "content/str_interp.tdr");
    let symbol = file_symbol(&db, project, file)
      .value(&db)
      .expect("file_symbol should return a resource symbol");
    let result = evaluate_resource(&db, symbol);
    let obj = result.value(&db).expect("should produce an object");
    let result_field = obj
      .get_owned_field(&db, "result")
      .expect("should have result field");
    let str_obj = (result_field.as_ref() as &dyn Any)
      .downcast_ref::<TdrStrObj>()
      .expect("result should be TdrStrObj");
    assert_eq!(str_obj.value(&db), "hello 42");
  }

  fn get_num_field(
    db: &crate::TypedownDatabase,
    obj: &Box<dyn crate::types::TdrObjectLike>,
    field: &str,
  ) -> f64 {
    let field_obj = obj.get_owned_field(db, field).expect("should have field");
    (field_obj.as_ref() as &dyn Any)
      .downcast_ref::<TdrNumObj>()
      .unwrap_or_else(|| panic!("{field} should be TdrNumObj"))
      .value(db)
  }

  fn get_bool_field(
    db: &crate::TypedownDatabase,
    obj: &Box<dyn crate::types::TdrObjectLike>,
    field: &str,
  ) -> bool {
    let field_obj = obj.get_owned_field(db, field).expect("should have field");
    (field_obj.as_ref() as &dyn Any)
      .downcast_ref::<TdrBoolObj>()
      .unwrap_or_else(|| panic!("{field} should be TdrBoolObj"))
      .value(db)
  }

  // 1 + 2 evaluates to 3
  #[test]
  fn binary_add_evaluates_to_sum() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "content/binary_valid.tdr");
    let symbol = file_symbol(&db, project, file).value(&db).unwrap();
    let obj = evaluate_resource(&db, symbol).value(&db).unwrap();
    assert_eq!(get_num_field(&db, &obj, "result"), 3.0);
  }

  // -, *, /, %, ** all produce the expected numeric result
  #[test]
  fn arithmetic_ops_evaluate_correctly() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "content/arithmetic_ops.tdr");
    let symbol = file_symbol(&db, project, file).value(&db).unwrap();
    let obj = evaluate_resource(&db, symbol).value(&db).unwrap();
    assert_eq!(get_num_field(&db, &obj, "sub"), 7.0);
    assert_eq!(get_num_field(&db, &obj, "mul"), 12.0);
    assert_eq!(get_num_field(&db, &obj, "div"), 2.5);
    assert_eq!(get_num_field(&db, &obj, "mod"), 1.0);
    assert_eq!(get_num_field(&db, &obj, "pow"), 256.0);
  }

  // Unary - negates the number
  #[test]
  fn unary_negation_evaluates_correctly() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "content/unary_valid.tdr");
    let symbol = file_symbol(&db, project, file).value(&db).unwrap();
    let obj = evaluate_resource(&db, symbol).value(&db).unwrap();
    assert_eq!(get_num_field(&db, &obj, "result"), -42.0);
  }

  // <, >, ==, !=, <=, >= all produce bool results for numeric operands
  #[test]
  fn comparison_ops_evaluate_correctly() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "content/comparison_ops.tdr");
    let symbol = file_symbol(&db, project, file).value(&db).unwrap();
    let obj = evaluate_resource(&db, symbol).value(&db).unwrap();
    assert!(get_bool_field(&db, &obj, "lt"));
    assert!(get_bool_field(&db, &obj, "gt"));
    assert!(get_bool_field(&db, &obj, "eq"));
    assert!(get_bool_field(&db, &obj, "ne"));
    assert!(get_bool_field(&db, &obj, "le"));
    assert!(get_bool_field(&db, &obj, "ge"));
  }

  // && and || produce the expected bool result
  #[test]
  fn logical_ops_evaluate_correctly() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "content/logical_ops.tdr");
    let symbol = file_symbol(&db, project, file).value(&db).unwrap();
    let obj = evaluate_resource(&db, symbol).value(&db).unwrap();
    assert!(!get_bool_field(&db, &obj, "and_false"));
    assert!(get_bool_field(&db, &obj, "or_true"));
  }

  // Unary + is identity; ~ is logical not (falsy: null/false; truthy: everything else)
  #[test]
  fn unary_extras_evaluate_correctly() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "content/unary_extras.tdr");
    let symbol = file_symbol(&db, project, file).value(&db).unwrap();
    let obj = evaluate_resource(&db, symbol).value(&db).unwrap();
    assert_eq!(get_num_field(&db, &obj, "pos"), 5.0);
    assert!(get_bool_field(&db, &obj, "logical_not_false"));
    assert!(!get_bool_field(&db, &obj, "logical_not_true"));
    assert!(!get_bool_field(&db, &obj, "logical_not_num"));
  }

  // String comparison operators work lexicographically
  #[test]
  fn str_comparison_evaluates_correctly() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "content/str_comparison.tdr");
    let symbol = file_symbol(&db, project, file).value(&db).unwrap();
    let obj = evaluate_resource(&db, symbol).value(&db).unwrap();
    assert!(get_bool_field(&db, &obj, "eq"));
    assert!(get_bool_field(&db, &obj, "ne"));
    assert!(get_bool_field(&db, &obj, "lt"));
  }

  // list[n] evaluates the list and returns the nth element
  #[test]
  fn list_index_evaluates_correctly() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "content/list_index.tdr");
    let symbol = file_symbol(&db, project, file).value(&db).unwrap();
    let obj = evaluate_resource(&db, symbol).value(&db).unwrap();
    assert_eq!(get_num_field(&db, &obj, "result"), 20.0);
  }

  // out-of-bounds index on list and string evaluates to undefined and emits a diagnostic
  #[test]
  fn index_out_of_bounds_emits_diagnostic() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "content/index_oob.tdr");
    let (hir, _) = lower_frontmatter(&db, project, file);
    let hir = hir.unwrap();

    // Extract field HIRs from the top-level mapping
    let HirValueKind::Mapping(entries) = hir.kind(&db) else {
      panic!("expected mapping at top level");
    };
    let field_hirs: std::collections::HashMap<_, _> =
      entries.into_iter().collect();

    let list_result = evaluate_node(&db, field_hirs["list_oob"]);
    assert!(list_result.value(&db).is_none(), "list OOB should be undefined");
    assert!(
      list_result.diagnostics(&db).iter().any(|d| matches!(
        d,
        typedown_types::diagnostic::Diagnostic::IndexOutOfBounds { index: 99, length: 3, .. }
      )),
      "expected IndexOutOfBounds(99, 3) for list, got: {:?}",
      list_result.diagnostics(&db)
    );

    let str_result = evaluate_node(&db, field_hirs["str_oob"]);
    assert!(str_result.value(&db).is_none(), "string OOB should be undefined");
    assert!(
      str_result.diagnostics(&db).iter().any(|d| matches!(
        d,
        typedown_types::diagnostic::Diagnostic::IndexOutOfBounds { index: 99, length: 5, .. }
      )),
      "expected IndexOutOfBounds(99, 5) for string, got: {:?}",
      str_result.diagnostics(&db)
    );
  }

  // string[n] returns the nth character as a string
  #[test]
  fn str_index_evaluates_correctly() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "content/str_index.tdr");
    let symbol = file_symbol(&db, project, file).value(&db).unwrap();
    let obj = evaluate_resource(&db, symbol).value(&db).unwrap();
    let result = obj.get_owned_field(&db, "result").unwrap();
    let str_obj = (result.as_ref() as &dyn Any)
      .downcast_ref::<TdrStrObj>()
      .expect("result should be TdrStrObj");
    assert_eq!(str_obj.value(&db), "e");
  }

  // Tag expressions like !str "Alice" strip the tag and evaluate the inner value
  #[test]
  fn tag_expr_strips_tag_and_evaluates_inner() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "content/tag_expr.tdr");
    let symbol = file_symbol(&db, project, file).value(&db).unwrap();
    let obj = evaluate_resource(&db, symbol).value(&db).unwrap();
    let name = obj.get_owned_field(&db, "name").unwrap();
    let name_str = (name.as_ref() as &dyn Any)
      .downcast_ref::<TdrStrObj>()
      .expect("name should be TdrStrObj");
    assert_eq!(name_str.value(&db), "Alice");
    assert_eq!(get_num_field(&db, &obj, "age"), 30.0);
  }
}
