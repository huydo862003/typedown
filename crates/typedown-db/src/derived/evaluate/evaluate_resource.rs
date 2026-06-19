//! Evaluate a resource file into typed objects

use typedown_macros::query_derived;
use typedown_syntax::ast::{AstNode, SourceFile};

use crate::derived::hir::lower_expr;
use crate::derived::parse_file::parse_file;
use crate::derived::typechecker::get_node_type::get_node_type;
use crate::derived::typechecker::typecheck::typecheck;
use crate::types::{HirValue, ResourceResult, Symbol, SymbolKind, TdrObjectLike};
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

  // Typecheck the resource
  let typecheck_result = typecheck(db, hir);
  diagnostics.extend(typecheck_result.diagnostics(db).iter().cloned());

  // Construct the object using the type's constructor
  let obj = construct_from_hir(db, hir);

  ResourceResult::new(db, obj, diagnostics)
}

fn construct_from_hir(db: &TypedownDatabase, hir: HirValue) -> Option<Box<dyn TdrObjectLike>> {
  let type_result = get_node_type(db, hir);
  let typ = type_result.typ(db)?;
  typ.construct(db, hir)
}

#[cfg(test)]
mod tests {
  use std::any::Any;

  use crate::{
    derived::evaluate::evaluate_resource::evaluate_resource,
    derived::name_resolver::file_symbol::file_symbol,
    fixtures::load_vault_fixture,
    types::{TdrProductObj, TdrProductType, TdrStrObj},
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
    let product = (obj.as_ref() as &dyn Any)
      .downcast_ref::<TdrProductObj>()
      .expect("should be TdrProductObj");
    let fields = product.fields(&db);
    let name = fields.get("name").expect("should have name");
    let name_str = (name.as_ref() as &dyn Any)
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
}
