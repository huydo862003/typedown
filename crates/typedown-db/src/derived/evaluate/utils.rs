use std::any::Any;
use std::collections::HashMap;

use crate::TypedownDatabase;
use typedown_types::either::Either;
use crate::derived::evaluate::evaluate_node::evaluate_node;
use crate::derived::evaluate::evaluate_type::resolve_property_descriptor;
use crate::derived::get_builtin_types::get_schema_type;
use crate::derived::evaluate::evaluate_resource::evaluate_resource;
use crate::derived::name_resolver::file_symbol::file_symbol;
use crate::derived::name_resolver::referee::referee;
use crate::derived::typechecker::get_node_type::get_node_type;
use crate::types::{
  BuiltinMacroKind, File, HirValue, HirValueKind, InterpolatedPart, SymbolKind, TdrBoolObj,
  MemberType, TdrDictObj, TdrFuncObj, TdrListObj, TdrListType, TdrMathObj, TdrNumObj, TdrObjectLike,
  TdrProductObj, TdrProductType, TdrSchemaType, TdrStrObj, TdrTypeLike, TypeMember,
  TypeMemberDescriptors,
};
use typedown_types::diagnostic::Diagnostic;

pub(crate) fn construct_from_hir(
  db: &TypedownDatabase,
  hir: HirValue,
  diagnostics: &mut Vec<Diagnostic>,
) -> Option<Box<dyn TdrObjectLike>> {
  match hir.kind(db) {
    // self evaluates to the current file's resource object
    HirValueKind::Ident(name) if name == "self" => {
      let project = hir.project(db);
      let file = hir.file(db);
      let symbol = file_symbol(db, project, file).value(db)?;
      return evaluate_resource(db, symbol).value(db);
    }
    // Tag expressions: the tag is a type hint for the typechecker; evaluation strips it
    HirValueKind::Tag { inner, .. } => {
      return evaluate_node(db, *inner).value(db);
    }
    // Field access: obj.field
    HirValueKind::Binary { op, left, right } if op == "." => {
      if let HirValueKind::Ident(field_name) = right.kind(db) {
        let this = evaluate_node(db, *left).value(db)?;
        return this.lookup_field(db, &field_name);
      }
    }
    // Arithmetic, comparison, and logical binary operators
    HirValueKind::Binary { op, left, right } => {
      return evaluate_binary(db, &op, *left, *right);
    }
    // Unary operators
    HirValueKind::Unary { op, operand } => {
      return evaluate_unary(db, &op, *operand);
    }
    // Index access: list[n] or dict["key"]
    HirValueKind::Index { expr, indices } => {
      return evaluate_index(db, *expr, indices, diagnostics);
    }
    HirValueKind::Call { callee, args } => {
      match callee.kind(db) {
        // Method call: obj.method(args)
        HirValueKind::Binary { op, left, right } if op == "." => {
          if let HirValueKind::Ident(method_name) = right.kind(db) {
            let this = evaluate_node(db, *left).value(db)?;
            let func_obj = this.lookup_method(db, &method_name)?;
            let arg_objs: Vec<_> = args
              .into_iter()
              .filter_map(|arg| evaluate_node(db, arg).value(db))
              .collect();
            return func_obj.call(db, this, arg_objs);
          }
        }
        // Macro calls: pass raw HIR args (macros need project context from HIR)
        _ => {
          let resolved = referee(db, *callee);
          if let Some(symbol) = resolved.value(db) {
            if let SymbolKind::BuiltinMacro(kind) = symbol.kind(db) {
              return construct_macro(db, kind, args);
            }
          }
          // Plain function call: evaluate callee, check if it's a function, call it
          let callee_obj = evaluate_node(db, *callee).value(db)?;
          if let Some(func_obj) = (callee_obj.as_ref() as &dyn Any).downcast_ref::<TdrFuncObj>() {
            let func_obj = func_obj.clone();
            let arg_objs: Vec<_> = args
              .into_iter()
              .filter_map(|arg| evaluate_node(db, arg).value(db))
              .collect();
            return func_obj.call(db, callee_obj, arg_objs);
          }
        }
      }
    }
    _ => {}
  }

  // Normal construction: convert HIR to args, then call construct
  let type_result = get_node_type(db, hir);
  let typ = type_result.typ(db)?;
  match hir.kind(db) {
    HirValueKind::Str(val) => typ.construct(db, vec![Box::new(TdrStrObj::new(db, val))]),
    HirValueKind::Num(val) => {
      let num: f64 = val.parse().unwrap_or(0.0);
      typ.construct(db, vec![Box::new(TdrNumObj::new(db, num))])
    }
    HirValueKind::Bool(val) => typ.construct(db, vec![Box::new(TdrBoolObj::new(db, val))]),
    HirValueKind::Math(val) => typ.construct(db, vec![Box::new(TdrMathObj::new(db, val))]),
    HirValueKind::Interpolated(parts) => {
      let obj = evaluate_interpolated(db, parts)?;
      typ.construct(db, vec![obj])
    }
    HirValueKind::Sequence(items) => {
      if (typ.as_ref() as &dyn Any).downcast_ref::<TdrListType>().is_some() {
        let hir_items = items.into_iter().map(Either::Left).collect();
        return Some(Box::new(TdrListObj::new(db, hir_items)));
      }
      let args: Vec<_> = items
        .into_iter()
        .filter_map(|item| evaluate_node(db, item).value(db))
        .collect();
      typ.construct(db, args)
    }
    HirValueKind::Mapping(entries) => evaluate_mapping(db, &*typ, entries),
    HirValueKind::Markdown(parts) => evaluate_interpolated(db, parts),
    _ => None,
  }
}

fn evaluate_unary(
  db: &TypedownDatabase,
  op: &str,
  operand: HirValue,
) -> Option<Box<dyn TdrObjectLike>> {
  let operand_obj = evaluate_node(db, operand).value(db)?;
  match op {
    "-" | "+" => {
      let num = (operand_obj.as_ref() as &dyn Any).downcast_ref::<TdrNumObj>()?;
      let val = num.value(db);
      let result = match op {
        "-" => -val,
        "+" => val,
        _ => unreachable!(),
      };
      Some(Box::new(TdrNumObj::new(db, result)))
    }
    // Logical not: only null and false are falsy, everything else is truthy
    "~" => {
      let is_falsy = (operand_obj.as_ref() as &dyn Any)
        .downcast_ref::<TdrBoolObj>()
        .map_or(false, |b| !b.value(db));
      Some(Box::new(TdrBoolObj::new(db, is_falsy)))
    }
    _ => None,
  }
}

fn evaluate_binary(
  db: &TypedownDatabase,
  op: &str,
  left: HirValue,
  right: HirValue,
) -> Option<Box<dyn TdrObjectLike>> {
  let left_obj = evaluate_node(db, left).value(db)?;
  let right_obj = evaluate_node(db, right).value(db)?;
  match op {
    "+" | "-" | "*" | "/" | "%" | "**" => {
      let lnum = (left_obj.as_ref() as &dyn Any).downcast_ref::<TdrNumObj>()?;
      let rnum = (right_obj.as_ref() as &dyn Any).downcast_ref::<TdrNumObj>()?;
      let lval = lnum.value(db);
      let rval = rnum.value(db);
      let result = match op {
        "+" => lval + rval,
        "-" => lval - rval,
        "*" => lval * rval,
        "/" => lval / rval,
        "%" => lval % rval,
        "**" => lval.powf(rval),
        _ => unreachable!(),
      };
      Some(Box::new(TdrNumObj::new(db, result)))
    }
    "==" | "!=" | "<" | ">" | "<=" | ">=" => {
      let result = compare_objects(db, op, left_obj.as_ref(), right_obj.as_ref())?;
      Some(Box::new(TdrBoolObj::new(db, result)))
    }
    "&&" | "||" => {
      let lbool = (left_obj.as_ref() as &dyn Any).downcast_ref::<TdrBoolObj>()?;
      let rbool = (right_obj.as_ref() as &dyn Any).downcast_ref::<TdrBoolObj>()?;
      let result = match op {
        "&&" => lbool.value(db) && rbool.value(db),
        "||" => lbool.value(db) || rbool.value(db),
        _ => unreachable!(),
      };
      Some(Box::new(TdrBoolObj::new(db, result)))
    }
    _ => None,
  }
}

fn compare_objects(
  db: &TypedownDatabase,
  op: &str,
  left: &dyn TdrObjectLike,
  right: &dyn TdrObjectLike,
) -> Option<bool> {
  if let (Some(lnum), Some(rnum)) = (
    (left as &dyn Any).downcast_ref::<TdrNumObj>(),
    (right as &dyn Any).downcast_ref::<TdrNumObj>(),
  ) {
    let lval = lnum.value(db);
    let rval = rnum.value(db);
    return Some(match op {
      "==" => lval == rval,
      "!=" => lval != rval,
      "<" => lval < rval,
      ">" => lval > rval,
      "<=" => lval <= rval,
      ">=" => lval >= rval,
      _ => return None,
    });
  }
  if let (Some(lstr), Some(rstr)) = (
    (left as &dyn Any).downcast_ref::<TdrStrObj>(),
    (right as &dyn Any).downcast_ref::<TdrStrObj>(),
  ) {
    let lval = lstr.value(db);
    let rval = rstr.value(db);
    return Some(match op {
      "==" => lval == rval,
      "!=" => lval != rval,
      "<" => lval < rval,
      ">" => lval > rval,
      "<=" => lval <= rval,
      ">=" => lval >= rval,
      _ => return None,
    });
  }
  if let (Some(lbool), Some(rbool)) = (
    (left as &dyn Any).downcast_ref::<TdrBoolObj>(),
    (right as &dyn Any).downcast_ref::<TdrBoolObj>(),
  ) {
    return Some(match op {
      "==" => lbool.value(db) == rbool.value(db),
      "!=" => lbool.value(db) != rbool.value(db),
      _ => return None,
    });
  }
  // Fallback: use ID-based comparison for any two objects
  let lid = left.as_id();
  let rid = right.as_id();
  Some(match op {
    "==" => lid == rid,
    "!=" => lid != rid,
    "<" => lid < rid,
    ">" => lid > rid,
    "<=" => lid <= rid,
    ">=" => lid >= rid,
    _ => return None,
  })
}

fn evaluate_index(
  db: &TypedownDatabase,
  expr: HirValue,
  indices: Vec<HirValue>,
  diagnostics: &mut Vec<Diagnostic>,
) -> Option<Box<dyn TdrObjectLike>> {
  if indices.len() != 1 {
    return None;
  }
  let index_hir = indices[0];
  let container = evaluate_node(db, expr).value(db)?;
  let index_obj = evaluate_node(db, index_hir).value(db)?;

  if let Some(list) = (container.as_ref() as &dyn Any).downcast_ref::<TdrListObj>() {
    let num = (index_obj.as_ref() as &dyn Any).downcast_ref::<TdrNumObj>()?;
    let idx = num.value(db) as usize;
    let len = list.len(db);
    if idx >= len {
      let node = index_hir.node(db);
      diagnostics.push(Diagnostic::IndexOutOfBounds {
        index: idx,
        length: len,
        start_offset: node.offset(),
        end_offset: node.offset() + node.text_len(),
      });
      return None;
    }
    return list.get(db, idx);
  }
  if let Some(dict) = (container.as_ref() as &dyn Any).downcast_ref::<TdrDictObj>() {
    let key = (index_obj.as_ref() as &dyn Any).downcast_ref::<TdrStrObj>()?;
    return dict.get_owned_field(db, &key.value(db));
  }
  if let Some(str_obj) = (container.as_ref() as &dyn Any).downcast_ref::<TdrStrObj>() {
    let num = (index_obj.as_ref() as &dyn Any).downcast_ref::<TdrNumObj>()?;
    let idx = num.value(db) as usize;
    let chars: Vec<char> = str_obj.value(db).chars().collect();
    if idx >= chars.len() {
      let node = index_hir.node(db);
      diagnostics.push(Diagnostic::IndexOutOfBounds {
        index: idx,
        length: chars.len(),
        start_offset: node.offset(),
        end_offset: node.offset() + node.text_len(),
      });
      return None;
    }
    return Some(Box::new(TdrStrObj::new(db, chars[idx].to_string())));
  }
  None
}

fn construct_macro(
  db: &TypedownDatabase,
  kind: BuiltinMacroKind,
  args: Vec<HirValue>,
) -> Option<Box<dyn TdrObjectLike>> {
  match kind {
    BuiltinMacroKind::Fref => construct_fref(db, args),
  }
}

fn evaluate_interpolated(
  db: &TypedownDatabase,
  parts: Vec<InterpolatedPart>,
) -> Option<Box<dyn TdrObjectLike>> {
  let mut val = String::new();
  for part in parts {
    match part {
      InterpolatedPart::Literal(lit) => val.push_str(&lit),
      InterpolatedPart::Expr(expr) => {
        let obj = evaluate_node(db, expr).value(db)?;
        let to_string_fn = obj.lookup_method(db, "to_string")?;
        let str_obj = to_string_fn.call(db, obj, vec![])?;
        let str_val = (str_obj.as_ref() as &dyn Any).downcast_ref::<TdrStrObj>()?;
        val.push_str(&str_val.value(db));
      }
    }
  }
  Some(Box::new(TdrStrObj::new(db, val)))
}

// Evaluate mapping as an object of type `typ`
fn evaluate_mapping(
  db: &TypedownDatabase,
  typ: &dyn TdrTypeLike,
  entries: Vec<(String, HirValue)>,
) -> Option<Box<dyn TdrObjectLike>> {
  // Schema type
  if (typ as &dyn Any).downcast_ref::<TdrSchemaType>().is_some() {
    let properties_entries = match entries.iter().find(|(key, _)| key == "properties") {
      Some((_, props_hir)) => match props_hir.kind(db) {
        HirValueKind::Mapping(entries) => entries,
        _ => return None,
      },
      None => vec![],
    };
    let mut fields = HashMap::new();
    for (prop_name, prop_hir) in properties_entries {
      if prop_name.starts_with('_') && prop_name != "_type" && prop_name != "_label" && prop_name != "_content" {
        fields.insert(
          prop_name,
          TypeMember::new(db, MemberType::Never, TypeMemberDescriptors::empty()),
        );
        continue;
      }
      if let Some((member_type, descriptors)) =
        resolve_property_descriptor(db, prop_hir, &mut vec![])
      {
        fields.insert(prop_name, TypeMember::new(db, member_type, descriptors));
      }
    }
    return Some(Box::new(TdrProductType::new(
      db,
      None,
      Box::new(get_schema_type(db)),
      fields,
    )));
  }

  // Product type
  if let Some(product_typ) = (typ as &dyn Any).downcast_ref::<TdrProductType>() {
    let mut fields = HashMap::new();
    for (key, val_hir) in entries {
      if key == "_type" {
        continue;
      }
      fields.insert(key, Either::Left(val_hir));
    }
    return Some(Box::new(TdrProductObj::new(
      db,
      Box::new(product_typ.clone()) as Box<dyn crate::types::TdrTypeLike>,
      fields,
    )));
  }


  let dict_entries: HashMap<_, _> = entries
    .into_iter()
    .map(|(k, v)| (k, Either::Left(v)))
    .collect();
  Some(Box::new(TdrDictObj::new(db, dict_entries)))
}

// fref("file.tdr") evaluates to the target resource's object
fn construct_fref(db: &TypedownDatabase, args: Vec<HirValue>) -> Option<Box<dyn TdrObjectLike>> {
  if args.len() != 1 {
    return None;
  }
  let arg = args[0];
  let path_str = match arg.kind(db) {
    HirValueKind::Str(val) => val,
    _ => return None,
  };

  let project = arg.project(db);
  let handles = project.handles(db);
  let root_dir = project.root_dir(db);
  let target_path = root_dir.join(&path_str);

  let target_handle = handles.get(&target_path)?.clone();
  let target_file = File::new(db, target_handle);
  let target_symbol = file_symbol(db, project, target_file).value(db)?;

  evaluate_resource(db, target_symbol).value(db)
}
