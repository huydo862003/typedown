//! Evaluate a schema symbol to extract the type it defines.

use std::collections::HashMap;

use typedown_macros::query_derived;
use typedown_syntax::ast::{AstNode, SourceFile};
use typedown_types::diagnostic::Diagnostic;

use crate::derived::get_builtin_types::{
  get_bool_type, get_date_type, get_datetime_type, get_dict_type, get_link_type, get_list_type,
  get_num_type, get_schema_type, get_str_type, get_time_type, get_type_type,
};
use crate::derived::hir::lower_expr;
use crate::derived::name_resolver::referee::referee;
use crate::derived::parse_file::parse_file;
use crate::derived::typechecker::typecheck::typecheck;
use crate::inputs::File;
use crate::types::Project;
use crate::types::{
  BuiltinSchemaKind, HirValue, HirValueKind, MemberType, Symbol, SymbolKind, TdrProductType,
  TdrTypeLike, TypeMember, TypeMemberDescriptors, TypeResult,
};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn evaluate_type(db: &TypedownDatabase, symbol: Symbol) -> TypeResult {
  match symbol.kind(db) {
    SymbolKind::BuiltinSchema(kind) => {
      let typ: Box<dyn TdrTypeLike> = match kind {
        BuiltinSchemaKind::Str => Box::new(get_str_type(db)),
        BuiltinSchemaKind::Num => Box::new(get_num_type(db)),
        BuiltinSchemaKind::Bool => Box::new(get_bool_type(db)),
        BuiltinSchemaKind::Date => Box::new(get_date_type(db)),
        BuiltinSchemaKind::DateTime => Box::new(get_datetime_type(db)),
        BuiltinSchemaKind::Time => Box::new(get_time_type(db)),
        BuiltinSchemaKind::List => Box::new(get_list_type(db)),
        BuiltinSchemaKind::Dict => Box::new(get_dict_type(db)),
        BuiltinSchemaKind::Link => Box::new(get_link_type(db)),
        BuiltinSchemaKind::Schema => Box::new(get_schema_type(db)),
        BuiltinSchemaKind::TypeType => Box::new(get_type_type(db)),
      };
      TypeResult::new(db, Some(typ), vec![])
    }
    SymbolKind::UserDefinedSchema(project, file) => {
      evaluate_user_defined_schema(db, symbol.name(db), project, file)
    }
    SymbolKind::UserDefinedResource(_, _) => {
      // Resources are not types
      TypeResult::new(db, None, vec![])
    }
  }
}

fn evaluate_user_defined_schema(
  db: &TypedownDatabase,
  schema_name: String,
  project: Project,
  file: File,
) -> TypeResult {
  let mut diagnostics = vec![];

  // Parse file and lower frontmatter to HIR
  let parse_result = parse_file(db, project, file);
  let root = parse_result.ast(db);
  let source_file = match SourceFile::cast(root) {
    Some(sf) => sf,
    None => return TypeResult::new(db, None, vec![]),
  };
  let mapping = match source_file.frontmatter().and_then(|fm| fm.mapping()) {
    Some(m) => m,
    None => return TypeResult::new(db, None, vec![]),
  };
  let hir = lower_expr(db, project, file, mapping.syntax().clone());

  // Typecheck the schema file (diagnostics not propagated to callers)
  let _ = typecheck(db, hir);

  // Extract entries from the frontmatter mapping
  let entries = match hir.kind(db) {
    HirValueKind::Mapping(entries) => entries,
    _ => return TypeResult::new(db, None, diagnostics),
  };

  // Find the "properties" entry
  let properties_hir = entries.iter().find(|(key, _)| key == "properties");
  let properties_entries = match properties_hir {
    Some((_, props_hir)) => match props_hir.kind(db) {
      HirValueKind::Mapping(entries) => entries,
      _ => {
        let node = props_hir.node(db);
        diagnostics.push(Diagnostic::FieldTypeMismatch {
          field: "properties".to_string(),
          expected: "mapping".to_string(),
          start_offset: node.offset(),
          end_offset: node.offset() + node.text_len(),
        });
        return TypeResult::new(db, None, diagnostics);
      }
    },
    None => {
      // Schema with no properties: empty product type
      return TypeResult::new(
        db,
        Some(Box::new(TdrProductType::new(
          db,
          Some(schema_name.clone()),
          Box::new(get_schema_type(db)),
          HashMap::new(),
        ))),
        diagnostics,
      );
    }
  };

  // The resulting fields of the product/schema type
  let mut fields = HashMap::new();

  // Loop through the declared props
  for (prop_name, prop_hir) in properties_entries {
    if let Some((member_type, descriptors)) =
      resolve_property_descriptor(db, prop_hir, &mut diagnostics)
    {
      fields.insert(
        prop_name.clone(),
        TypeMember::new(db, member_type, descriptors),
      );
    }
  }

  TypeResult::new(
    db,
    Some(Box::new(TdrProductType::new(
      db,
      Some(schema_name),
      Box::new(get_schema_type(db)),
      fields,
    ))),
    diagnostics,
  )
}

// Process a property descriptor like `{ type: string, required: true }`
pub(crate) fn resolve_property_descriptor(
  db: &TypedownDatabase,
  hir: HirValue,
  diagnostics: &mut Vec<Diagnostic>,
) -> Option<(MemberType, TypeMemberDescriptors)> {
  let entries = match hir.kind(db) {
    HirValueKind::Mapping(entries) => entries,
    _ => return None,
  };

  let mut field_type: Option<MemberType> = None;
  let mut descriptors = TypeMemberDescriptors::empty();

  for (key, value) in &entries {
    match key.as_str() {
      "type" => {
        field_type = resolve_type_member(db, *value, diagnostics);
      }
      "required" => {
        if let HirValueKind::Bool(false) = value.kind(db) {
          descriptors |= TypeMemberDescriptors::OPTIONAL;
        }
      }
      _ => {}
    }
  }

  field_type.map(|typ| (typ, descriptors))
}

fn resolve_type_member(
  db: &TypedownDatabase,
  hir: HirValue,
  diagnostics: &mut Vec<Diagnostic>,
) -> Option<MemberType> {
  match hir.kind(db) {
    // Simple type reference like `type: string`
    HirValueKind::Ident(_) => {
      let resolved = referee(db, hir);
      match resolved.value(db) {
        Some(symbol) => {
          let result = evaluate_type(db, symbol);
          diagnostics.extend(result.diagnostics(db).iter().cloned());
          result.typ(db).map(MemberType::Simple)
        }
        None => {
          let node = hir.node(db);
          diagnostics.push(Diagnostic::UnresolvedSchema {
            name: node.text(),
            start_offset: node.offset(),
            end_offset: node.offset() + node.text_len(),
          });
          None
        }
      }
    }
    // Union type like `type: [string, number]`
    HirValueKind::Sequence(items) => {
      let mut members = vec![];
      for item in items {
        if let Some(member_type) = resolve_type_member(db, item, diagnostics) {
          members.push(TypeMember::new(
            db,
            member_type,
            TypeMemberDescriptors::empty(),
          ));
        }
      }
      if members.is_empty() {
        None
      } else {
        Some(MemberType::Sum(members))
      }
    }
    // Inline object like `type: { name: { type: string }, age: { type: number } }`
    // Each entry is a property descriptor with `type` and optional `required`
    HirValueKind::Mapping(entries) => {
      let mut fields = HashMap::new();
      for (key, value_hir) in entries {
        if let Some((member_type, descriptors)) =
          resolve_property_descriptor(db, value_hir, diagnostics)
        {
          fields.insert(key.clone(), TypeMember::new(db, member_type, descriptors));
        }
      }
      Some(MemberType::Simple(Box::new(TdrProductType::new(
        db,
        None,
        Box::new(get_type_type(db)),
        fields,
      ))))
    }
    _ => {
      let node = hir.node(db);
      diagnostics.push(Diagnostic::UnresolvedSchema {
        name: node.text(),
        start_offset: node.offset(),
        end_offset: node.offset() + node.text_len(),
      });
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use std::any::Any;

  use std::collections::HashMap;
  use std::path::PathBuf;

  use typedown_syntax::ast::{AstNode, SourceFile};

  use crate::{
    QueryStorage, TypedownDatabase,
    derived::evaluate::evaluate_type::evaluate_type,
    derived::get_builtin_types::*,
    derived::hir::lower_expr,
    derived::name_resolver::file_symbol::file_symbol,
    derived::parse_file::parse_file,
    derived::typechecker::get_node_type::get_node_type,
    fixtures::load_vault_fixture,
    inputs::{File, FileHandle},
    types::{
      BuiltinSchemaKind, HirValue, HirValueKind, MemberType, Project, Symbol, SymbolKind,
      TdrBoolObj, TdrDictObj, TdrListObj, TdrNumObj, TdrObjectType, TdrProductObj, TdrProductType,
      TdrStrObj, TdrTypeLike, TdrTypeType, TypeMember, TypeMemberDescriptors,
    },
  };

  fn make_db() -> TypedownDatabase {
    TypedownDatabase {
      storage: QueryStorage::default(),
    }
  }

  #[test]
  fn evaluate_type_builtin_schema_returns_schema_type() {
    let db = make_db();
    let symbol = Symbol::new(
      &db,
      SymbolKind::BuiltinSchema(BuiltinSchemaKind::Schema),
      "Schema".to_string(),
    );

    let result = evaluate_type(&db, symbol);

    let expected = Some(Box::new(get_schema_type(&db)) as Box<dyn TdrTypeLike>);
    assert!(
      result.typ(&db) == expected,
      "builtin Schema symbol should evaluate to TdrSchemaType"
    );
    assert!(
      result.diagnostics(&db).is_empty(),
      "expected no diagnostics"
    );
  }

  #[test]
  fn evaluate_user_defined_schema_returns_product_type() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "schemas/Person.tdr");
    let symbol = file_symbol(&db, project, file).value(&db).unwrap();

    let result = evaluate_type(&db, symbol);

    let typ = result.typ(&db).expect("should return a type");
    assert!(
      (typ.as_ref() as &dyn Any)
        .downcast_ref::<TdrProductType>()
        .is_some(),
      "user-defined schema should evaluate to TdrProductType"
    );
  }

  #[test]
  fn evaluate_user_defined_schema_has_declared_fields() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "schemas/Person.tdr");
    let symbol = file_symbol(&db, project, file).value(&db).unwrap();

    let result = evaluate_type(&db, symbol);
    let typ = result.typ(&db).unwrap();
    let product = (typ.as_ref() as &dyn Any)
      .downcast_ref::<TdrProductType>()
      .unwrap();
    let fields = product.fields(&db);
    assert!(fields.contains_key("name"), "should have 'name' field");
    assert!(fields.contains_key("age"), "should have 'age' field");
  }

  #[test]
  fn evaluate_type_no_properties_returns_empty_product() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "schemas/NoProperties.tdr");
    let symbol = file_symbol(&db, project, file).value(&db).unwrap();

    let result = evaluate_type(&db, symbol);
    let typ = result.typ(&db).expect("should return a type");
    let product = (typ.as_ref() as &dyn Any)
      .downcast_ref::<TdrProductType>()
      .unwrap();
    assert!(
      product.fields(&db).is_empty(),
      "schema with no properties should have empty fields"
    );
  }

  #[test]
  fn evaluate_type_wrong_properties_type_has_diagnostics() {
    let (db, project, file) =
      load_vault_fixture("evaluate/my_vault", "schemas/WrongProperties.tdr");
    let symbol = file_symbol(&db, project, file).value(&db).unwrap();

    let result = evaluate_type(&db, symbol);
    assert!(
      !result.diagnostics(&db).is_empty(),
      "schema with non-mapping properties should have diagnostics"
    );
  }

  #[test]
  fn evaluate_type_wrong_property_descriptor_has_diagnostics() {
    let (db, project, file) =
      load_vault_fixture("evaluate/my_vault", "schemas/WrongPropertyDescriptor.tdr");
    let symbol = file_symbol(&db, project, file).value(&db).unwrap();

    let result = evaluate_type(&db, symbol);
    assert!(
      !result.diagnostics(&db).is_empty(),
      "schema with unresolved property type should have diagnostics: {:?}",
      result.diagnostics(&db)
    );
  }

  #[test]
  fn display_name_builtin_types() {
    let db = make_db();

    assert_eq!(get_str_type(&db).display_name(&db), "string");
    assert_eq!(get_num_type(&db).display_name(&db), "number");
    assert_eq!(get_bool_type(&db).display_name(&db), "boolean");
    assert_eq!(get_date_type(&db).display_name(&db), "date");
    assert_eq!(get_datetime_type(&db).display_name(&db), "datetime");
    assert_eq!(get_time_type(&db).display_name(&db), "time");
    assert_eq!(get_list_type(&db).display_name(&db), "list");
    assert_eq!(get_dict_type(&db).display_name(&db), "dict");
    assert_eq!(get_link_type(&db).display_name(&db), "link");
    assert_eq!(get_type_type(&db).display_name(&db), "type");
    assert_eq!(get_object_type(&db).display_name(&db), "object");
    assert_eq!(get_schema_type(&db).display_name(&db), "Schema");
  }

  #[test]
  fn display_name_instantiated_list() {
    let db = make_db();

    let list_str = instantiate_type(
      &db,
      Box::new(get_list_type(&db)),
      vec![Box::new(get_str_type(&db))],
    );
    assert_eq!(list_str.typ(&db).display_name(&db), "list[string]");
  }

  #[test]
  fn display_name_instantiated_dict() {
    let db = make_db();

    let dict_str_num = instantiate_type(
      &db,
      Box::new(get_dict_type(&db)),
      vec![Box::new(get_str_type(&db)), Box::new(get_num_type(&db))],
    );
    assert_eq!(
      dict_str_num.typ(&db).display_name(&db),
      "dict[string, number]"
    );
  }

  #[test]
  fn display_name_user_defined_schema() {
    let (db, project, file) = load_vault_fixture("typecheck/my_vault", "schemas/Person.tdr");
    let symbol = file_symbol(&db, project, file).value(&db).unwrap();

    let result = evaluate_type(&db, symbol);
    let typ = result.typ(&db).unwrap();
    assert_eq!(typ.display_name(&db), "Person");
  }

  #[test]
  fn display_name_anonymous_product() {
    let db = make_db();

    let product = TdrProductType::new(
      &db,
      None,
      Box::new(get_type_type(&db)),
      HashMap::from([(
        "name".to_string(),
        TypeMember::new(
          &db,
          MemberType::Simple(Box::new(get_str_type(&db))),
          TypeMemberDescriptors::empty(),
        ),
      )]),
    );
    assert_eq!(product.display_name(&db), "{ name: string }");
  }

  // Helper to create an HirValue from a frontmatter string
  fn make_hir(db: &TypedownDatabase, content: &str) -> HirValue {
    let file = File::new(db, FileHandle::Content(content.to_string()));
    let project = Project::new(db, PathBuf::new(), HashMap::new());
    let parse_result = parse_file(db, project, file);
    let root = parse_result.ast(db);
    let source_file = SourceFile::cast(root).unwrap();
    let mapping = source_file.frontmatter().unwrap().mapping().unwrap();
    lower_expr(db, project, file, mapping.syntax().clone())
  }

  // Helper to get a specific field's HirValue from a frontmatter mapping
  fn get_field_hir(db: &TypedownDatabase, hir: HirValue, field: &str) -> HirValue {
    match hir.kind(db) {
      HirValueKind::Mapping(entries) => entries.into_iter().find(|(k, _)| k == field).unwrap().1,
      _ => panic!("expected mapping"),
    }
  }

  #[test]
  fn construct_str() {
    let db = make_db();
    let hir = make_hir(
      &db,
      r#"---
val: "hello"
---"#,
    );
    let val_hir = get_field_hir(&db, hir, "val");

    let str_type = get_str_type(&db);
    let obj = str_type.construct(&db, val_hir).expect("should construct");
    let str_obj = (obj.as_ref() as &dyn Any)
      .downcast_ref::<TdrStrObj>()
      .expect("should be TdrStrObj");
    assert_eq!(str_obj.value(&db), "hello");
  }

  #[test]
  fn construct_num() {
    let db = make_db();
    let hir = make_hir(
      &db,
      r#"---
val: 42
---"#,
    );
    let val_hir = get_field_hir(&db, hir, "val");

    let num_type = get_num_type(&db);
    let obj = num_type.construct(&db, val_hir).expect("should construct");
    let num_obj = (obj.as_ref() as &dyn Any)
      .downcast_ref::<TdrNumObj>()
      .expect("should be TdrNumObj");
    assert_eq!(num_obj.value(&db), 42.0);
  }

  #[test]
  fn construct_bool() {
    let db = make_db();
    let hir = make_hir(
      &db,
      r#"---
val: true
---"#,
    );
    let val_hir = get_field_hir(&db, hir, "val");

    let bool_type = get_bool_type(&db);
    let obj = bool_type.construct(&db, val_hir).expect("should construct");
    let bool_obj = (obj.as_ref() as &dyn Any)
      .downcast_ref::<TdrBoolObj>()
      .expect("should be TdrBoolObj");
    assert!(bool_obj.value(&db));
  }

  #[test]
  fn construct_str_returns_none_for_wrong_hir() {
    let db = make_db();
    let hir = make_hir(
      &db,
      r#"---
val: 42
---"#,
    );
    let val_hir = get_field_hir(&db, hir, "val");

    let str_type = get_str_type(&db);
    assert!(
      str_type.construct(&db, val_hir).is_none(),
      "str construct should reject Num HIR"
    );
  }

  // Product type construct from a mapping
  #[test]
  fn construct_product() {
    let (db, project, file) = load_vault_fixture("typecheck/my_vault", "content/valid_person.tdr");
    let root = parse_file(&db, project, file).ast(&db);
    let mapping = SourceFile::cast(root)
      .unwrap()
      .frontmatter()
      .unwrap()
      .mapping()
      .unwrap();
    let hir = lower_expr(&db, project, file, mapping.syntax().clone());

    let type_result = get_node_type(&db, hir);
    let typ = type_result.typ(&db).unwrap();
    let obj = typ.construct(&db, hir).expect("should construct product");
    let product_obj = (obj.as_ref() as &dyn Any)
      .downcast_ref::<TdrProductObj>()
      .expect("should be TdrProductObj");
    let fields = product_obj.fields(&db);
    let name_obj = fields.get("name").expect("should have name field");
    let name_str = (name_obj.as_ref() as &dyn Any)
      .downcast_ref::<TdrStrObj>()
      .expect("name should be TdrStrObj");
    assert_eq!(name_str.value(&db), "Alice");
  }

  // List construct from a sequence
  #[test]
  fn construct_list() {
    let db = make_db();
    let hir = make_hir(
      &db,
      r#"---
val: [1, 2, 3]
---"#,
    );
    let val_hir = get_field_hir(&db, hir, "val");

    let list_num = instantiate_type(
      &db,
      Box::new(get_list_type(&db)),
      vec![Box::new(get_num_type(&db))],
    );
    let obj = list_num
      .typ(&db)
      .construct(&db, val_hir)
      .expect("should construct list");
    let list_obj = (obj.as_ref() as &dyn Any)
      .downcast_ref::<TdrListObj>()
      .expect("should be TdrListObj");
    let items = list_obj.items(&db);
    assert_eq!(items.len(), 3);
    let first = (items[0].as_ref() as &dyn Any)
      .downcast_ref::<TdrNumObj>()
      .expect("first item should be TdrNumObj");
    assert_eq!(first.value(&db), 1.0);
  }

  // Schema construct via evaluate_type
  #[test]
  fn construct_schema() {
    let (db, project, file) = load_vault_fixture("typecheck/my_vault", "schemas/Person.tdr");
    let root = parse_file(&db, project, file).ast(&db);
    let mapping = SourceFile::cast(root)
      .unwrap()
      .frontmatter()
      .unwrap()
      .mapping()
      .unwrap();
    let hir = lower_expr(&db, project, file, mapping.syntax().clone());

    let schema_type = get_schema_type(&db);
    let obj = schema_type
      .construct(&db, hir)
      .expect("should construct schema");
    let product_type = (obj.as_ref() as &dyn Any)
      .downcast_ref::<TdrProductType>()
      .expect("schema construct should produce TdrProductType");
    assert!(
      product_type.fields(&db).contains_key("name"),
      "should have name field"
    );
    assert!(
      product_type.fields(&db).contains_key("age"),
      "should have age field"
    );
  }

  // TdrObjectType construct falls back to dict for mappings without _type
  #[test]
  fn construct_object_type_fallback_to_dict() {
    let db = make_db();
    let hir = make_hir(
      &db,
      r#"---
name: "Alice"
age: 42
---"#,
    );
    let val_hir = get_field_hir(&db, hir, "name");

    // Calling construct on ObjectType for a string value delegates to TdrStrType
    let obj_type = TdrObjectType::get(&db);
    let obj = obj_type
      .construct(&db, val_hir)
      .expect("should construct via delegation");
    let str_obj = (obj.as_ref() as &dyn Any)
      .downcast_ref::<TdrStrObj>()
      .expect("should delegate to TdrStrType and produce TdrStrObj");
    assert_eq!(str_obj.value(&db), "Alice");
  }

  // TdrTypeType construct produces a TdrProductType from a schema mapping
  #[test]
  fn construct_type_type() {
    let (db, project, file) = load_vault_fixture("typecheck/my_vault", "schemas/Person.tdr");
    let root = parse_file(&db, project, file).ast(&db);
    let mapping = SourceFile::cast(root)
      .unwrap()
      .frontmatter()
      .unwrap()
      .mapping()
      .unwrap();
    let hir = lower_expr(&db, project, file, mapping.syntax().clone());

    let type_type = TdrTypeType::get(&db);
    let obj = type_type
      .construct(&db, hir)
      .expect("should construct type from schema");
    let product_type = (obj.as_ref() as &dyn Any)
      .downcast_ref::<TdrProductType>()
      .expect("TdrTypeType construct should produce TdrProductType");
    assert!(
      product_type.fields(&db).contains_key("name"),
      "should have name field"
    );
    assert!(
      product_type.fields(&db).contains_key("age"),
      "should have age field"
    );
  }

  // TdrTypeType construct returns None for non-schema mappings
  #[test]
  fn construct_type_type_rejects_non_schema() {
    let (db, project, file) = load_vault_fixture("typecheck/my_vault", "content/valid_person.tdr");
    let root = parse_file(&db, project, file).ast(&db);
    let mapping = SourceFile::cast(root)
      .unwrap()
      .frontmatter()
      .unwrap()
      .mapping()
      .unwrap();
    let hir = lower_expr(&db, project, file, mapping.syntax().clone());

    let type_type = TdrTypeType::get(&db);
    assert!(
      type_type.construct(&db, hir).is_none(),
      "TdrTypeType construct should reject non-schema mappings"
    );
  }

  // link[Person] accepts a schema type
  #[test]
  fn link_instantiate_with_schema() {
    let (db, project, file) = load_vault_fixture("typecheck/my_vault", "schemas/Person.tdr");
    let symbol = file_symbol(&db, project, file).value(&db).unwrap();
    let person_type = evaluate_type(&db, symbol).typ(&db).unwrap();

    let result = instantiate_type(&db, Box::new(get_link_type(&db)), vec![person_type]);
    assert!(
      result.diagnostics(&db).is_empty(),
      "link[Person] should have no diagnostics: {:?}",
      result.diagnostics(&db)
    );
    assert_eq!(result.typ(&db).display_name(&db), "link[Person]");
  }

  // link[string] rejects a non-schema type
  #[test]
  fn link_instantiate_rejects_non_schema() {
    let db = make_db();
    let result = instantiate_type(
      &db,
      Box::new(get_link_type(&db)),
      vec![Box::new(get_str_type(&db))],
    );
    assert!(
      !result.diagnostics(&db).is_empty(),
      "link[string] should have diagnostics"
    );
  }
}
