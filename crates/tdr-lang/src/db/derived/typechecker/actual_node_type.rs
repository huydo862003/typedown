//! Tracked query to get the type of a HIR value

use std::collections::HashMap;
use tdr_incremental::Id;

use crate::db::TypedownDatabase;
use crate::db::derived::evaluate::evaluate_type::evaluate_type;
use crate::db::derived::get_builtin_types::{
  get_bool_type, get_date_type, get_datetime_type, get_list_type, get_math_type, get_num_type,
  get_str_type, get_time_type, get_type_type, instantiate_type,
};
use crate::db::derived::name_resolver::file_symbol::file_symbol;
use crate::db::derived::name_resolver::referee::referee;
use crate::db::derived::typechecker::get_symbol_type::get_symbol_type;
use crate::db::types::derived::object_system::{
  is_valid_iso_date, is_valid_iso_datetime, is_valid_iso_time,
};
use crate::db::types::{
  BuiltinMacroKind, HirValue, HirValueKind, LiteralValue, MemberType, SymbolKind, TdrProductType,
  TdrStrType, TdrTypeEnum, TdrTypeLike, TypeMember, TypeMemberDescriptors, TypeResult,
};
use crate::db::utils::lower_file;
use crate::syntax::diagnostic::Diagnostic;
use tdr_incremental::QueryDatabase;
use tdr_macros::query_derived;

// Infer the type of an HIR
// This function never relies on the declared type of the hir (it can rely on the declared type of the referenced hir)
// It always guesses based on the structure of the hir alone
#[query_derived]
pub fn actual_node_type(db: &TypedownDatabase, hir: HirValue) -> TypeResult {
  match hir.kind(db) {
    HirValueKind::Str(ref val) => {
      // Deduce the most specific string subtype from the value's format
      let typ: TdrTypeEnum = if is_valid_iso_datetime(val) {
        get_datetime_type(db).into()
      } else if is_valid_iso_date(val) {
        get_date_type(db).into()
      } else if is_valid_iso_time(val) {
        get_time_type(db).into()
      } else {
        get_str_type(db).into()
      };
      TypeResult::new(db, Some(typ), vec![])
    }
    HirValueKind::Interpolated(_) => TypeResult::new(db, Some(get_str_type(db).into()), vec![]),
    HirValueKind::Num(_) => TypeResult::new(db, Some(get_num_type(db).into()), vec![]),
    HirValueKind::Bool(_) => TypeResult::new(db, Some(get_bool_type(db).into()), vec![]),
    HirValueKind::Null => TypeResult::new(db, None, vec![]),
    HirValueKind::Ident(ref name) if name == "self" => get_self_type(db, hir),
    HirValueKind::Ident(_) => {
      let resolved = referee(db, hir);
      match resolved.value(db) {
        Some(symbol) => get_symbol_type(db, symbol),
        None => TypeResult::new(db, None, vec![]),
      }
    }
    HirValueKind::Mapping(entries) => get_mapping_type(db, hir, entries),
    HirValueKind::Sequence(items) => get_sequence_type(db, items),
    HirValueKind::Call { callee, args } => get_call_type(db, *callee, args),
    HirValueKind::Index { expr, indices } => get_index_type(db, *expr, indices),
    HirValueKind::Tag { tag, .. } => get_tag_type(db, *tag),
    HirValueKind::Unary { op, operand } => get_unary_type(db, &op, *operand),
    HirValueKind::Binary { op, left, right } => get_binary_type(db, &op, *left, *right),
    HirValueKind::Math(_) => TypeResult::new(db, Some(get_math_type(db).into()), vec![]),
    HirValueKind::Markdown(_) => TypeResult::new(db, Some(get_str_type(db).into()), vec![]),
  }
}

/// Narrow a TdrTypeEnum to the most specific MemberType based on the value's HIR kind
fn narrow_field_member_type(db: &TypedownDatabase, hir: &HirValue, typ: TdrTypeEnum) -> MemberType {
  /// Collect narrowed member types from a list of HIR values into TypeMember arms
  fn collect_narrowed_arms(
    db: &TypedownDatabase,
    values: impl Iterator<Item = HirValue>,
  ) -> Vec<TypeMember> {
    values
      .filter_map(|val| {
        // Extract the loose type
        let typ = actual_node_type(db, val).typ(db)?;
        // Narrow the loose type into a type member
        let member = narrow_field_member_type(db, &val, typ);
        Some(TypeMember::new(db, member, TypeMemberDescriptors::empty()))
      })
      .collect()
  }

  match hir.kind(db) {
    // String literals -> the literal string member type
    HirValueKind::Str(val) => MemberType::Literal(LiteralValue::Str(val)),
    // Number literals -> the literal number member type
    HirValueKind::Num(val) => MemberType::Literal(LiteralValue::Num(val)),
    // Boolean literals -> the literal boolean member type
    HirValueKind::Bool(val) => MemberType::Literal(LiteralValue::Bool(val)),

    // Sequence literals -> narrow down the items and collect into ListOfSum
    HirValueKind::Sequence(items) => {
      let arms = collect_narrowed_arms(db, items.into_iter());
      if arms.is_empty() {
        // No arms -> List of Never so it can match any list
        MemberType::ListOfSum(vec![TypeMember::new(
          db,
          MemberType::Never,
          TypeMemberDescriptors::empty(),
        )])
      } else {
        MemberType::ListOfSum(arms)
      }
    }

    // Mapping literals -> narrow down the items and collect into DictOfSum
    HirValueKind::Mapping(inner_entries) => {
      // Product is already most narrow, so no need to narrow further
      if typ.as_tdr_product_type().is_some() {
        MemberType::Simple(typ)
      } else {
        let arms = collect_narrowed_arms(db, inner_entries.into_iter().map(|(_, val)| val));
        if arms.is_empty() {
          // No arms -> Dict of Never so it can match any dict
          MemberType::DictOfSum(vec![TypeMember::new(
            db,
            MemberType::Never,
            TypeMemberDescriptors::empty(),
          )])
        } else {
          MemberType::DictOfSum(arms)
        }
      }
    }

    _ => MemberType::Simple(typ),
  }
}

/// Helper to get the type of a mapping
/// NOTE: Always return a product type because they can be generalized to a dict type
fn get_mapping_type(
  db: &TypedownDatabase,
  _hir: HirValue,
  entries: Vec<(String, HirValue)>,
) -> TypeResult {
  let mut diagnostics = vec![];
  let mut fields = HashMap::new();
  for (key, value_hir) in entries {
    if key == "_type" {
      continue;
    }
    let field_result = actual_node_type(db, value_hir);
    diagnostics.extend(field_result.diagnostics(db).iter().cloned());
    if let Some(typ) = field_result.typ(db) {
      let member_type = narrow_field_member_type(db, &value_hir, typ);
      fields.insert(
        key,
        TypeMember::new(db, member_type, TypeMemberDescriptors::empty()),
      );
    }
  }
  TypeResult::new(
    db,
    Some(TdrProductType::new(db, None, get_type_type(db).into(), fields).into()),
    diagnostics,
  )
}

/// Resolve a tag expression like `!Person { name: "John" }`
fn get_tag_type(db: &TypedownDatabase, tag: HirValue) -> TypeResult {
  let resolved = referee(db, tag);
  match resolved.value(db) {
    Some(symbol) => evaluate_type(db, symbol),
    None => {
      let node = tag.node(db);
      TypeResult::new(
        db,
        None,
        vec![Diagnostic::UnresolvedSchema {
          name: node.text(),
          start_offset: node.offset(),
          end_offset: node.offset() + node.text_len(),
        }],
      )
    }
  }
}

/// Helper to get the return type of a unary expression
fn get_unary_type(db: &TypedownDatabase, op: &str, operand: HirValue) -> TypeResult {
  let operand_result = actual_node_type(db, operand);
  let diagnostics = operand_result.diagnostics(db).clone();

  match op {
    // Arithmetic negation and plus: returns number
    "-" | "+" => TypeResult::new(db, Some(get_num_type(db).into()), diagnostics),
    // Logical not: accepts any type, returns boolean
    "~" => TypeResult::new(db, Some(get_bool_type(db).into()), diagnostics),
    _ => TypeResult::new(db, None, diagnostics),
  }
}

/// Helper to get the return type of a binary expression
fn get_binary_type(db: &TypedownDatabase, op: &str, left: HirValue, right: HirValue) -> TypeResult {
  // Field access such as `obj.field`
  if op == "." {
    let left_result = actual_node_type(db, left);
    let mut diagnostics = left_result.diagnostics(db).clone();
    let left_type = match left_result.typ(db) {
      Some(typ) => typ,
      None => return TypeResult::new(db, None, diagnostics),
    };
    let field_name = match right.kind(db) {
      HirValueKind::Ident(name) => name,
      _ => return TypeResult::new(db, None, diagnostics),
    };
    return match left_type.lookup_field_type(db, &field_name) {
      Some(typ) => TypeResult::new(db, Some(typ), diagnostics),
      None => {
        let node = right.node(db);
        diagnostics.push(Diagnostic::UnknownField {
          field: field_name,
          on_type: left_type.display_name(db),
          start_offset: node.offset(),
          end_offset: node.offset() + node.text_len(),
        });
        TypeResult::new(db, None, diagnostics)
      }
    };
  }

  let left_result = actual_node_type(db, left);
  let right_result = actual_node_type(db, right);
  let mut diagnostics = left_result.diagnostics(db).clone();
  diagnostics.extend(right_result.diagnostics(db).iter().cloned());

  match op {
    // Arithmetic: returns number
    "+" | "-" | "*" | "/" | "%" | "**" => {
      TypeResult::new(db, Some(get_num_type(db).into()), diagnostics)
    }
    // Comparison: returns boolean
    "==" | "!=" | "<" | ">" | "<=" | ">=" => {
      TypeResult::new(db, Some(get_bool_type(db).into()), diagnostics)
    }
    // Logical: returns boolean
    "&&" | "||" => TypeResult::new(db, Some(get_bool_type(db).into()), diagnostics),
    _ => TypeResult::new(db, None, diagnostics),
  }
}

/// Helper to get the type of a sequence
/// NOTE: This function infers element type as the most general type across all items, then instantiates list[elem]
fn get_sequence_type(db: &TypedownDatabase, items: Vec<HirValue>) -> TypeResult {
  let mut diagnostics = vec![];
  let mut elem_type: Option<TdrTypeEnum> = None;

  for item in items {
    let item_result = actual_node_type(db, item);
    diagnostics.extend(item_result.diagnostics(db).iter().cloned());

    if let Some(item_type) = item_result.typ(db) {
      elem_type = Some(match elem_type {
        None => item_type,
        Some(current) => find_common_supertype(db, current, item_type),
      });
    }
  }

  let list_type = match elem_type {
    Some(typ) => {
      let inst_result = instantiate_type(db, get_list_type(db).into(), vec![typ]);
      diagnostics.extend(inst_result.diagnostics(db).iter().cloned());
      Some(inst_result.typ(db))
    }
    None => Some(get_list_type(db).into()),
  };
  TypeResult::new(db, list_type, diagnostics)
}

/// Helper to get the type of a call expression
/// NOTE: This function only synthesizes the return type & arg checking is done by typecheck (not
/// tis function)
fn get_call_type(db: &TypedownDatabase, callee: HirValue, args: Vec<HirValue>) -> TypeResult {
  // Check if callee is a macro
  let resolved = referee(db, callee);
  if let Some(symbol) = resolved.value(db)
    && let SymbolKind::BuiltinMacro(kind) = symbol.kind(db)
  {
    return get_macro_call_type(db, kind, args);
  }

  let callee_result = actual_node_type(db, callee);
  let diagnostics = callee_result.diagnostics(db).clone();

  let callee_type = match callee_result.typ(db) {
    Some(typ) => typ,
    None => return TypeResult::new(db, None, diagnostics),
  };

  if let TdrTypeEnum::TdrFuncType(func) = &callee_type {
    let sig = func.signature(db);
    return TypeResult::new(db, Some(sig.ret(db)), diagnostics);
  }

  TypeResult::new(db, None, diagnostics)
}

fn get_macro_call_type(
  db: &TypedownDatabase,
  kind: BuiltinMacroKind,
  args: Vec<HirValue>,
) -> TypeResult {
  match kind {
    BuiltinMacroKind::Fref => get_fref_type(db, args),
  }
}

// fref("file.tdr") returns link[T] where T is the target file's schema type
fn get_fref_type(db: &TypedownDatabase, args: Vec<HirValue>) -> TypeResult {
  if args.len() != 1 {
    let node = args.first().map(|a| a.node(db));
    return TypeResult::new(
      db,
      None,
      vec![Diagnostic::WrongArgCount {
        expected: 1,
        got: args.len(),
        start_offset: node.as_ref().map_or(0, |n| n.offset()),
        end_offset: node.as_ref().map_or(0, |n| n.offset() + n.text_len()),
      }],
    );
  }
  let arg = args[0];
  let node = arg.node(db);
  let path_str = match arg.kind(db) {
    HirValueKind::Str(val) => val,
    _ => {
      return TypeResult::new(
        db,
        None,
        vec![Diagnostic::ArgTypeMismatch {
          expected: "string".to_string(),
          start_offset: node.offset(),
          end_offset: node.offset() + node.text_len(),
        }],
      );
    }
  };

  let project = arg.project(db);
  let files = project.files(db);
  let root_dir = project.root_dir(db);
  let target_path = root_dir.join(&path_str);

  let target_file = match files.get(&target_path) {
    Some(file) => *file,
    None => {
      return TypeResult::new(
        db,
        None,
        vec![Diagnostic::UnresolvedFileRef {
          path: path_str,
          start_offset: node.offset(),
          end_offset: node.offset() + node.text_len(),
        }],
      );
    }
  };
  let target_symbol = file_symbol(db, project, target_file);

  match target_symbol.value(db) {
    Some(sym) => {
      // Return the resource's type directly
      get_symbol_type(db, sym)
    }
    None => TypeResult::new(
      db,
      None,
      vec![Diagnostic::UnresolvedSchema {
        name: path_str,
        start_offset: node.offset(),
        end_offset: node.offset() + node.text_len(),
      }],
    ),
  }
}

/// Helper to get the type of an index expression
/// NOTE: This funcion only returns the result type & index checking is done by typecheck (not this
/// function)
fn get_index_type(db: &TypedownDatabase, expr: HirValue, indices: Vec<HirValue>) -> TypeResult {
  let expr_result = actual_node_type(db, expr);
  let mut diagnostics = expr_result.diagnostics(db).clone();

  let expr_type = match expr_result.typ(db) {
    Some(typ) => typ,
    None => return TypeResult::new(db, None, diagnostics),
  };

  // Type instantiation (e.g. `list[string]`, `dict[string, number]`)
  // Indices are type expressions, so we should resolve them as schemas, not values
  if expr_type.arity(db) > 0 {
    let mut arg_types = vec![];
    for idx_hir in indices {
      let resolved = referee(db, idx_hir);
      match resolved.value(db) {
        Some(symbol) => {
          let schema_result = evaluate_type(db, symbol);
          diagnostics.extend(schema_result.diagnostics(db).iter().cloned());
          match schema_result.typ(db) {
            Some(typ) => arg_types.push(typ),
            None => return TypeResult::new(db, None, diagnostics),
          }
        }
        None => {
          let node = idx_hir.node(db);
          diagnostics.push(Diagnostic::UnresolvedSchema {
            name: node.text(),
            start_offset: node.offset(),
            end_offset: node.offset() + node.text_len(),
          });
          return TypeResult::new(db, None, diagnostics);
        }
      }
    }
    let inst_result = instantiate_type(db, expr_type, arg_types);
    diagnostics.extend(inst_result.diagnostics(db).iter().cloned());
    return TypeResult::new(db, Some(inst_result.typ(db)), diagnostics);
  }

  // Element access on instantiated list
  if let TdrTypeEnum::TdrListType(list) = &expr_type {
    return TypeResult::new(db, list.elem(db), diagnostics);
  }

  // Element access on instantiated dict
  if let TdrTypeEnum::TdrDictType(dict) = &expr_type {
    return TypeResult::new(db, dict.value(db), diagnostics);
  }

  // Element access on string (returns single-character string)
  if expr_type.is_tdr_str_type() {
    return TypeResult::new(db, Some(TdrStrType::get(db).into()), diagnostics);
  }

  TypeResult::new(db, None, diagnostics)
}

/// Walk up the supertype chain from `a` until it is compatible with `b`
fn find_common_supertype(db: &TypedownDatabase, a: TdrTypeEnum, b: TdrTypeEnum) -> TdrTypeEnum {
  // If a already accepts b, a is general enough
  if a.is_compatible_with(db, &b) {
    return a;
  }
  // If b already accepts a, b is general enough
  if b.is_compatible_with(db, &a) {
    return b;
  }

  // Walk up a's supertype chain
  let mut current = a;
  loop {
    let super_type = current.get_supertype(db);
    if super_type.as_id() == current.as_id() {
      // Reached and used ObjectType
      return super_type;
    }
    if super_type.is_compatible_with(db, &b) {
      return super_type;
    }
    current = super_type;
  }
}

/// Return the type of `self` in the current file
/// If the file declares `_type: SomeSchema`, self has that schema's type
fn get_self_type(db: &TypedownDatabase, hir: HirValue) -> TypeResult {
  let project = hir.project(db);
  let file = hir.file(db);
  let (mapping_hir, _) = lower_file(db, project, file);
  let mapping_hir = match mapping_hir {
    Some(mapping_hir) => mapping_hir,
    None => return TypeResult::new(db, None, vec![]),
  };

  if let HirValueKind::Mapping(entries) = mapping_hir.kind(db) {
    for (key, val_hir) in entries {
      if key == "_type" {
        let resolved = referee(db, val_hir);
        return match resolved.value(db) {
          Some(symbol) => evaluate_type(db, symbol),
          // Unresolved _type is already caught by typecheck on the mapping itself
          None => TypeResult::new(db, None, vec![]),
        };
      }
    }
  }
  TypeResult::new(db, None, vec![])
}

#[cfg(test)]
mod tests {
  use crate::db::types::TdrTypeEnum;
  use std::{collections::HashMap, path::PathBuf, time::SystemTime};

  use crate::db::{
    QueryStorage, TypedownDatabase,
    derived::{
      get_builtin_types::get_schema_type, typechecker::actual_node_type::actual_node_type,
    },
    types::{File, FileHandle, Project},
    utils::lower_file,
  };

  fn vault_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/evaluate_schema/my_vault")
  }

  use crate::db::{
    fixtures::load_vault_fixture,
    types::{LiteralValue, MemberType},
  };

  #[test]
  fn infer_anonymous_mapping_narrows_literal_fields() {
    let (db, project, file) =
      load_vault_fixture("typecheck/narrow_vault", "content/anonymous_mapping.tdr");
    let (hir, _) = lower_file(&db, project, file);
    let hir = hir.expect("should parse");
    let type_result = actual_node_type(&db, hir);
    let typ = type_result.typ(&db).expect("should infer a type");
    let product = typ.as_tdr_product_type().expect("should be a product type");
    let fields = product.fields(&db);

    // String literal narrows to Literal(Str)
    let name_member = fields.get("name").expect("should have name field");
    assert!(
      matches!(name_member.typ(&db), MemberType::Literal(LiteralValue::Str(s)) if s == "Alice"),
      "name should be Literal(Str(\"Alice\")), got: {:?}",
      name_member.typ(&db)
    );

    // Num literal narrows to Literal(Num)
    let age_member = fields.get("age").expect("should have age field");
    assert!(
      matches!(age_member.typ(&db), MemberType::Literal(LiteralValue::Num(n)) if n == "30"),
      "age should be Literal(Num(\"30\")), got: {:?}",
      age_member.typ(&db)
    );

    // Bool literal narrows to Literal(Bool)
    let active_member = fields.get("active").expect("should have active field");
    assert!(
      matches!(
        active_member.typ(&db),
        MemberType::Literal(LiteralValue::Bool(true))
      ),
      "active should be Literal(Bool(true)), got: {:?}",
      active_member.typ(&db)
    );

    // Sequence ["a", 3] narrows to ListOfSum with 2 arms
    let tags_member = fields.get("tags").expect("should have tags field");
    match tags_member.typ(&db) {
      MemberType::ListOfSum(arms) => {
        assert_eq!(arms.len(), 2, "tags should have 2 arms, got {}", arms.len());
        assert!(
          matches!(arms[0].typ(&db), MemberType::Literal(LiteralValue::Str(s)) if s == "a"),
          "first arm should be Literal(Str(\"a\")), got: {:?}",
          arms[0].typ(&db)
        );
        assert!(
          matches!(arms[1].typ(&db), MemberType::Literal(LiteralValue::Num(n)) if n == "3"),
          "second arm should be Literal(Num(\"3\")), got: {:?}",
          arms[1].typ(&db)
        );
      }
      other => panic!("tags should be ListOfSum, got: {:?}", other),
    }
  }

  #[test]
  fn actual_node_type_of_schema_file_top_level_mapping_is_schema_type() {
    let vault = vault_root();
    let schema_file_path = vault.join("schemas/Person.tdr");

    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };

    let file = File::new(
      &db,
      FileHandle::Path(schema_file_path.clone(), SystemTime::UNIX_EPOCH),
    );
    let files = HashMap::from([(schema_file_path, file)]);
    let project = Project::new(&db, vault, files);

    let (hir, _) = lower_file(&db, project, file);
    let hir = hir.expect("schema file should have parseable frontmatter");
    let type_result = actual_node_type(&db, hir);

    let expected = Some(TdrTypeEnum::from(get_schema_type(&db)));
    assert!(
      type_result.typ(&db) == expected,
      "top-level mapping of a schema file should have type TdrSchemaType"
    );
    assert!(
      type_result.diagnostics(&db).is_empty(),
      "expected no diagnostics, got: {:?}",
      type_result.diagnostics(&db)
    );
  }
}
