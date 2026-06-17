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
use crate::derived::typechecker::get_node_type::get_node_type;
use crate::derived::typechecker::typecheck::typecheck;
use crate::types::{
  BuiltinSchemaKind, HirValue, HirValueKind, MemberType, Symbol, SymbolKind, TdrProductType,
  TdrTypeLike, TypeMember, TypeMemberDescriptors, TypeResult,
};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn evaluate_schema(db: &TypedownDatabase, symbol: Symbol) -> TypeResult {
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

  // Typecheck the schema file
  let typecheck_result = typecheck(db, hir);
  diagnostics.extend(typecheck_result.diagnostics(db).iter().cloned());

  // Verify the declared type is the schema metatype
  let type_result = get_node_type(db, hir);
  diagnostics.extend(type_result.diagnostics(db).iter().cloned());
  let schema_type = Box::new(get_schema_type(db)) as Box<dyn TdrTypeLike>;
  let is_schema = type_result.typ(db).is_some_and(|typ| typ == schema_type);
  if !is_schema {
    let node = hir.node(db);
    diagnostics.push(Diagnostic::UnresolvedSchema {
      name: "expected _type: Schema".to_string(),
      start_offset: node.offset(),
      end_offset: node.offset() + node.text_len(),
    });
    return TypeResult::new(db, None, diagnostics);
  }

  // Extract properties from the frontmatter mapping
  let entries = match hir.kind(db) {
    HirValueKind::Mapping(entries) => entries,
    _ => return TypeResult::new(db, None, diagnostics),
  };

  // Find the "properties" entry
  let properties_hir = entries.iter().find(|(key, _)| key == "properties");
  let properties_entries = match properties_hir {
    Some((_, props_hir)) => match props_hir.kind(db) {
      HirValueKind::Mapping(entries) => entries,
      _ => return TypeResult::new(db, None, diagnostics),
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
    // The descriptor of the prop
    let prop_entries = match prop_hir.kind(db) {
      HirValueKind::Mapping(entries) => entries,
      _ => continue,
    };

    let mut field_type: Option<MemberType> = None;
    let mut descriptors = TypeMemberDescriptors::empty();

    for (desc_key, desc_value) in &prop_entries {
      match desc_key.as_str() {
        // Process the declared "type"
        "type" => {
          field_type = resolve_type_member(db, *desc_value, &mut diagnostics);
        }
        // Process the descriptor "required"
        "required" => {
          if let HirValueKind::Bool(false) = desc_value.kind(db) {
            descriptors |= TypeMemberDescriptors::OPTIONAL;
          }
        }
        _ => {}
      }
    }

    // Create the field from field type and descriptor
    if let Some(member_type) = field_type {
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

// Resolve a type expression in a property descriptor to a MemberType
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
          let result = evaluate_schema(db, symbol);
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
    // Inline object like `type: { name: { type: string } }`
    HirValueKind::Mapping(entries) => {
      let mut fields = HashMap::new();
      for (key, value_hir) in entries {
        if let Some(member_type) = resolve_type_member(db, value_hir, diagnostics) {
          fields.insert(
            key.clone(),
            TypeMember::new(db, member_type, TypeMemberDescriptors::empty()),
          );
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
    derived::evaluate::evaluate_schema::evaluate_schema,
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
  fn evaluate_schema_builtin_schema_returns_schema_type() {
    let db = make_db();
    let symbol = Symbol::new(
      &db,
      SymbolKind::BuiltinSchema(BuiltinSchemaKind::Schema),
      "Schema".to_string(),
    );

    let result = evaluate_schema(&db, symbol);

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

    let result = evaluate_schema(&db, symbol);

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

    let result = evaluate_schema(&db, symbol);
    let typ = result.typ(&db).unwrap();
    let product = (typ.as_ref() as &dyn Any)
      .downcast_ref::<TdrProductType>()
      .unwrap();
    let fields = product.fields(&db);
    assert!(fields.contains_key("name"), "should have 'name' field");
    assert!(fields.contains_key("age"), "should have 'age' field");
  }

  #[test]
  fn evaluate_schema_not_a_schema_has_diagnostics() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "schemas/NotASchema.tdr");
    let symbol = file_symbol(&db, project, file).value(&db).unwrap();

    let result = evaluate_schema(&db, symbol);
    assert!(
      !result.diagnostics(&db).is_empty(),
      "schema with wrong _type should have diagnostics"
    );
  }

  #[test]
  fn evaluate_schema_no_properties_returns_empty_product() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "schemas/NoProperties.tdr");
    let symbol = file_symbol(&db, project, file).value(&db).unwrap();

    let result = evaluate_schema(&db, symbol);
    let typ = result.typ(&db).expect("should return a type");
    let product = (typ.as_ref() as &dyn Any)
      .downcast_ref::<TdrProductType>()
      .unwrap();
    assert!(
      product.fields(&db).is_empty(),
      "schema with no properties should have empty fields"
    );
    // Should still have diagnostics for missing required 'properties' field
    assert!(
      !result.diagnostics(&db).is_empty(),
      "should have diagnostics for missing properties: {:?}",
      result.diagnostics(&db)
    );
  }

  #[test]
  fn evaluate_schema_wrong_properties_type_has_diagnostics() {
    let (db, project, file) =
      load_vault_fixture("evaluate/my_vault", "schemas/WrongProperties.tdr");
    let symbol = file_symbol(&db, project, file).value(&db).unwrap();

    let result = evaluate_schema(&db, symbol);
    assert!(
      !result.diagnostics(&db).is_empty(),
      "schema with non-mapping properties should have diagnostics"
    );
  }

  #[test]
  fn evaluate_schema_wrong_property_descriptor_has_diagnostics() {
    let (db, project, file) =
      load_vault_fixture("evaluate/my_vault", "schemas/WrongPropertyDescriptor.tdr");
    let symbol = file_symbol(&db, project, file).value(&db).unwrap();

    let result = evaluate_schema(&db, symbol);
    assert!(
      !result.diagnostics(&db).is_empty(),
      "schema with unresolved property type should have diagnostics: {:?}",
      result.diagnostics(&db)
    );
  }
}
