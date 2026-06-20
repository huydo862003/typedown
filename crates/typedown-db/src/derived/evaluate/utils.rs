use crate::TypedownDatabase;
use crate::derived::evaluate::evaluate_resource::evaluate_resource;
use crate::derived::name_resolver::file_symbol::file_symbol;
use crate::derived::name_resolver::referee::referee;
use crate::derived::typechecker::get_node_type::get_node_type;
use crate::types::{BuiltinMacroKind, File, HirValue, HirValueKind, SymbolKind, TdrObjectLike};

pub(crate) fn construct_from_hir(
  db: &TypedownDatabase,
  hir: HirValue,
) -> Option<Box<dyn TdrObjectLike>> {
  match hir.kind(db) {
    // self evaluates to the current file's resource object
    HirValueKind::Ident(name) if name == "self" => {
      let project = hir.project(db);
      let file = hir.file(db);
      let symbol = file_symbol(db, project, file).value(db)?;
      return evaluate_resource(db, symbol).value(db);
    }
    // Field access: obj.field
    HirValueKind::Binary { op, left, right } if op == "." => {
      if let HirValueKind::Ident(field_name) = right.kind(db) {
        let this = construct_from_hir(db, *left)?;
        return this.lookup_field(db, &field_name);
      }
    }
    HirValueKind::Call { callee, args } => {
      match callee.kind(db) {
        // Method call: obj.method(args)
        HirValueKind::Binary { op, left, right } if op == "." => {
          if let HirValueKind::Ident(method_name) = right.kind(db) {
            let this = construct_from_hir(db, *left)?;
            let func_obj = this.lookup_method(db, &method_name)?;
            let arg_objs: Vec<_> = args
              .into_iter()
              .filter_map(|arg| construct_from_hir(db, arg))
              .collect();
            return func_obj.call(db, this, arg_objs);
          }
        }
        // Macro calls like fref("file.tdr")
        _ => {
          let resolved = referee(db, *callee);
          if let Some(symbol) = resolved.value(db) {
            if let SymbolKind::BuiltinMacro(kind) = symbol.kind(db) {
              return construct_macro(db, kind, args);
            }
          }
        }
      }
    }
    _ => {}
  }

  // Normal construction
  let type_result = get_node_type(db, hir);
  let typ = type_result.typ(db)?;
  typ.construct(db, hir)
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

  let result = evaluate_resource(db, target_symbol);
  result.value(db)
}
