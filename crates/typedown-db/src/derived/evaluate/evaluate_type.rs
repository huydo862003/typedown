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
  }
}

fn evaluate_user_defined_schema(
  db: &TypedownDatabase,
  schema_name: String,
  project: crate::types::Project,
  file: crate::inputs::File,
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
    Some(Box::new(TdrProductType::new(db, Some(schema_name), fields))),
    diagnostics,
  )
}

// Process a property descriptor like `{ type: string, required: true }`
fn resolve_property_descriptor(
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
        db, None, fields,
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

  use crate::{
    QueryStorage, TypedownDatabase,
    derived::evaluate::evaluate_type::evaluate_type,
    derived::get_builtin_types::get_schema_type,
    derived::name_resolver::file_symbol::file_symbol,
    fixtures::load_vault_fixture,
    types::{BuiltinSchemaKind, Symbol, SymbolKind, TdrProductType, TdrTypeLike},
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
    use crate::derived::get_builtin_types::*;
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
    use crate::derived::get_builtin_types::*;
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
    use crate::derived::get_builtin_types::*;
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
    use crate::derived::get_builtin_types::*;
    let db = make_db();

    let product = TdrProductType::new(
      &db,
      None,
      std::collections::HashMap::from([(
        "name".to_string(),
        crate::types::TypeMember::new(
          &db,
          crate::types::MemberType::Simple(Box::new(get_str_type(&db))),
          crate::types::TypeMemberDescriptors::empty(),
        ),
      )]),
    );
    assert_eq!(product.display_name(&db), "{ name: string }");
  }
}
