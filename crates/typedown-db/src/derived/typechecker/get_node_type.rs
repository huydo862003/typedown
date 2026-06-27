//! Tracked query to get the type of a HIR value

use std::any::Any;
use std::collections::HashMap;

use crate::derived::evaluate::evaluate_type::evaluate_type;
use crate::derived::get_builtin_types::{
  get_bool_type, get_date_type, get_datetime_type, get_list_type, get_math_type, get_num_type,
  get_str_type, get_time_type, get_type_type, instantiate_type,
};
use crate::derived::name_resolver::file_symbol::file_symbol;
use crate::derived::name_resolver::referee::referee;
use crate::derived::typechecker::get_symbol_type::get_symbol_type;
use crate::types::derived::object_system::datetime::utils::{
  is_valid_iso_date, is_valid_iso_datetime, is_valid_iso_time,
};
use crate::types::{
  BuiltinMacroKind, HirValue, HirValueKind, MemberType, SymbolKind, TdrDictType, TdrFuncType,
  TdrListType, TdrProductType, TdrStrType, TdrTypeLike, TypeMember, TypeMemberDescriptors,
  TypeResult,
};
use crate::utils::lower_file;
use crate::{QueryDatabase, TypedownDatabase};
use typedown_macros::query_derived;
use typedown_types::diagnostic::Diagnostic;

#[query_derived]
pub fn get_node_type(db: &TypedownDatabase, hir: HirValue) -> TypeResult {
  match hir.kind(db) {
    HirValueKind::Str(ref val) => {
      // Deduce the most specific string subtype from the value's format
      let typ: Box<dyn TdrTypeLike> = if is_valid_iso_datetime(val) {
        Box::new(get_datetime_type(db))
      } else if is_valid_iso_date(val) {
        Box::new(get_date_type(db))
      } else if is_valid_iso_time(val) {
        Box::new(get_time_type(db))
      } else {
        Box::new(get_str_type(db))
      };
      TypeResult::new(db, Some(typ), vec![])
    }
    HirValueKind::Interpolated(_) => TypeResult::new(db, Some(Box::new(get_str_type(db))), vec![]),
    HirValueKind::Num(_) => TypeResult::new(db, Some(Box::new(get_num_type(db))), vec![]),
    HirValueKind::Bool(_) => TypeResult::new(db, Some(Box::new(get_bool_type(db))), vec![]),
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
    HirValueKind::Math(_) => TypeResult::new(db, Some(Box::new(get_math_type(db))), vec![]),
    HirValueKind::Markdown(_) => TypeResult::new(db, Some(Box::new(get_str_type(db))), vec![]),
  }
}

/// Helper to get the type of a mapping
/// NOTE: Always return a product type, if _type is not given
/// Can be generalized to a dict type
fn get_mapping_type(
  db: &TypedownDatabase,
  _hir: HirValue,
  entries: Vec<(String, HirValue)>,
) -> TypeResult {
  // If _type is present, resolve and evaluate the schema
  for (key, value_hir) in &entries {
    if key == "_type" {
      let resolved = referee(db, *value_hir);
      if let Some(symbol) = resolved.value(db) {
        return evaluate_type(db, symbol);
      }
      let node = value_hir.node(db);
      return TypeResult::new(
        db,
        None,
        vec![Diagnostic::UnresolvedSchema {
          name: node.text(),
          start_offset: node.offset(),
          end_offset: node.offset() + node.text_len(),
        }],
      );
    }
  }

  // No _type: infer a product type from the entries
  let mut diagnostics = vec![];
  let mut fields = HashMap::new();
  for (key, value_hir) in entries {
    let field_result = get_node_type(db, value_hir);
    diagnostics.extend(field_result.diagnostics(db).iter().cloned());
    if let Some(typ) = field_result.typ(db) {
      fields.insert(
        key,
        TypeMember::new(db, MemberType::Simple(typ), TypeMemberDescriptors::empty()),
      );
    }
  }
  TypeResult::new(
    db,
    Some(Box::new(TdrProductType::new(
      db,
      None,
      Box::new(get_type_type(db)),
      fields,
    ))),
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
  let operand_result = get_node_type(db, operand);
  let diagnostics = operand_result.diagnostics(db).clone();

  match op {
    // Arithmetic negation and plus: returns number
    "-" | "+" => TypeResult::new(db, Some(Box::new(get_num_type(db))), diagnostics),
    // Logical not: accepts any type, returns boolean
    "~" => TypeResult::new(db, Some(Box::new(get_bool_type(db))), diagnostics),
    _ => TypeResult::new(db, None, diagnostics),
  }
}

/// Helper to get the return type of a binary expression
fn get_binary_type(db: &TypedownDatabase, op: &str, left: HirValue, right: HirValue) -> TypeResult {
  // Field access such as `obj.field`
  if op == "." {
    let left_result = get_node_type(db, left);
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

  let left_result = get_node_type(db, left);
  let right_result = get_node_type(db, right);
  let mut diagnostics = left_result.diagnostics(db).clone();
  diagnostics.extend(right_result.diagnostics(db).iter().cloned());

  match op {
    // Arithmetic: returns number
    "+" | "-" | "*" | "/" | "%" | "**" => {
      TypeResult::new(db, Some(Box::new(get_num_type(db))), diagnostics)
    }
    // Comparison: returns boolean
    "==" | "!=" | "<" | ">" | "<=" | ">=" => {
      TypeResult::new(db, Some(Box::new(get_bool_type(db))), diagnostics)
    }
    // Logical: returns boolean
    "&&" | "||" => TypeResult::new(db, Some(Box::new(get_bool_type(db))), diagnostics),
    _ => TypeResult::new(db, None, diagnostics),
  }
}

/// Helper to get the type of a sequence
/// NOTE: This function infers element type as the most general type across all items, then instantiates list[elem]
fn get_sequence_type(db: &TypedownDatabase, items: Vec<HirValue>) -> TypeResult {
  let mut diagnostics = vec![];
  let mut elem_type: Option<Box<dyn TdrTypeLike>> = None;

  for item in items {
    let item_result = get_node_type(db, item);
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
      let inst_result = instantiate_type(db, Box::new(get_list_type(db)), vec![typ]);
      diagnostics.extend(inst_result.diagnostics(db).iter().cloned());
      Some(inst_result.typ(db))
    }
    None => Some(Box::new(get_list_type(db)) as Box<dyn crate::types::TdrTypeLike>),
  };
  TypeResult::new(db, list_type, diagnostics)
}

/// Helper to get the type of a call expression
/// NOTE: This function only synthesizes the return type & arg checking is done by typecheck (not
/// tis function)
fn get_call_type(db: &TypedownDatabase, callee: HirValue, args: Vec<HirValue>) -> TypeResult {
  // Check if callee is a macro
  let resolved = referee(db, callee);
  if let Some(symbol) = resolved.value(db) {
    if let SymbolKind::BuiltinMacro(kind) = symbol.kind(db) {
      return get_macro_call_type(db, kind, args);
    }
  }

  let callee_result = get_node_type(db, callee);
  let diagnostics = callee_result.diagnostics(db).clone();

  let callee_type = match callee_result.typ(db) {
    Some(typ) => typ,
    None => return TypeResult::new(db, None, diagnostics),
  };

  if let Some(func) = (callee_type.as_ref() as &dyn Any).downcast_ref::<TdrFuncType>() {
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
  let expr_result = get_node_type(db, expr);
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
  if let Some(list) = (expr_type.as_ref() as &dyn Any).downcast_ref::<TdrListType>() {
    return TypeResult::new(db, list.elem(db), diagnostics);
  }

  // Element access on instantiated dict
  if let Some(dict) = (expr_type.as_ref() as &dyn Any).downcast_ref::<TdrDictType>() {
    return TypeResult::new(db, dict.value(db), diagnostics);
  }

  // Element access on string (returns single-character string)
  if (expr_type.as_ref() as &dyn Any)
    .downcast_ref::<TdrStrType>()
    .is_some()
  {
    return TypeResult::new(db, Some(Box::new(TdrStrType::get(db))), diagnostics);
  }

  TypeResult::new(db, None, diagnostics)
}

/// Walk up the supertype chain from `a` until it is compatible with `b`
fn find_common_supertype(
  db: &TypedownDatabase,
  a: Box<dyn TdrTypeLike>,
  b: Box<dyn TdrTypeLike>,
) -> Box<dyn TdrTypeLike> {
  // If a already accepts b, a is general enough
  if a.is_compatible_with(db, b.as_ref()) {
    return a;
  }
  // If b already accepts a, b is general enough
  if b.is_compatible_with(db, a.as_ref()) {
    return b;
  }

  // Walk up a's supertype chain
  let mut current = a;
  loop {
    let super_type = current.get_supertype(db);
    if super_type.type_id() == current.type_id() && super_type.as_id() == current.as_id() {
      // Reached and used ObjectType
      return super_type;
    }
    if super_type.is_compatible_with(db, b.as_ref()) {
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
  use std::{collections::HashMap, path::PathBuf, time::SystemTime};

  use crate::{
    QueryStorage, TypedownDatabase,
    derived::{get_builtin_types::get_schema_type, typechecker::get_node_type::get_node_type},
    inputs::{File, FileHandle},
    types::{Project, TdrTypeLike},
    utils::lower_file,
  };

  fn vault_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/evaluate_schema/my_vault")
  }

  #[test]
  fn get_node_type_of_schema_file_top_level_mapping_is_schema_type() {
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
    let type_result = get_node_type(&db, hir);

    let expected = Some(Box::new(get_schema_type(&db)) as Box<dyn TdrTypeLike>);
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
