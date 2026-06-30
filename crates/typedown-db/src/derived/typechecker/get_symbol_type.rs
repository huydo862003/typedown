//! Tracked query to get the type of a symbol

use typedown_macros::query_derived;

use crate::derived::typechecker::infer_node_type::infer_node_type;
use crate::types::{Symbol, SymbolKind, TdrTypeType, TypeResult};
use crate::utils::lower_file;
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn get_symbol_type(db: &TypedownDatabase, symbol: Symbol) -> TypeResult {
  match symbol.kind(db) {
    // Schema symbols are types
    SymbolKind::BuiltinSchema(_) | SymbolKind::UserDefinedSchema(_, _) => {
      TypeResult::new(db, Some(TdrTypeType::get(db).into()), vec![])
    }
    // Resource symbols get their type from their frontmatter
    SymbolKind::UserDefinedResource(project, file) => {
      let (hir, _) = lower_file(db, project, file);
      match hir {
        Some(hir) => infer_node_type(db, hir),
        None => TypeResult::new(db, None, vec![]),
      }
    }
    // Macros don't have a type themselves
    SymbolKind::BuiltinMacro(_) => TypeResult::new(db, None, vec![]),
  }
}
