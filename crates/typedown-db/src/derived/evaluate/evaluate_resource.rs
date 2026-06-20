//! Evaluate a resource file into typed objects

use typedown_macros::query_derived;
use typedown_syntax::ast::{AstNode, SourceFile};

use crate::derived::evaluate::evaluate_node::evaluate_node;
use crate::derived::hir::lower_expr;
use crate::derived::parse_file::parse_file;
use crate::types::{ResourceResult, Symbol, SymbolKind};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn evaluate_resource(db: &TypedownDatabase, symbol: Symbol) -> ResourceResult {
  let (project, file) = match symbol.kind(db) {
    SymbolKind::UserDefinedResource(project, file) => (project, file),
    _ => return ResourceResult::new(db, None, vec![]),
  };

  let mut diagnostics = vec![];

  // Parse file and lower frontmatter to HIR
  let parse_result = parse_file(db, project, file);
  diagnostics.extend(parse_result.diagnostics(db).iter().cloned());
  let root = parse_result.ast(db);
  let source_file = match SourceFile::cast(root) {
    Some(sf) => sf,
    None => return ResourceResult::new(db, None, diagnostics),
  };
  let mapping = match source_file.frontmatter().and_then(|fm| fm.mapping()) {
    Some(m) => m,
    None => return ResourceResult::new(db, None, diagnostics),
  };
  let hir = lower_expr(db, project, file, mapping.syntax().clone());

  let node_result = evaluate_node(db, hir);
  diagnostics.extend(node_result.diagnostics(db).iter().cloned());

  ResourceResult::new(db, node_result.value(db), diagnostics)
}

#[cfg(test)]
mod tests {
  use std::any::Any;

  use crate::{
    derived::evaluate::evaluate_resource::evaluate_resource,
    derived::name_resolver::file_symbol::file_symbol,
    fixtures::load_vault_fixture,
    types::{TdrProductType, TdrStrObj},
  };

  #[test]
  fn evaluate_resource_valid_person() {
    let (db, project, file) =
      load_vault_fixture("evaluate/my_vault", "content/valid_person.tdr");
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
    let (db, project, file) =
      load_vault_fixture("evaluate/my_vault", "content/circular_a.tdr");
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
    let (db, project, file_a) =
      load_vault_fixture("evaluate/my_vault", "content/circular_a.tdr");
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
    let (db, project, file) =
      load_vault_fixture("evaluate/my_vault", "content/fref_prop.tdr");
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

  #[test]
  fn str_interp_evaluates_expr_parts() {
    let (db, project, file) =
      load_vault_fixture("evaluate/my_vault", "content/str_interp.tdr");
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
}
