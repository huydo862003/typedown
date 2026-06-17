//! Tracked query to get the type of a symbol

use typedown_macros::query_derived;

use crate::types::{Symbol, SymbolKind, TdrTypeType, TypeResult};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn get_symbol_type(db: &TypedownDatabase, symbol: Symbol) -> TypeResult {
  match symbol.kind(db) {
    // All schema symbols (builtin or user-defined) are types, so their type is TdrTypeType.
    SymbolKind::BuiltinSchema(_) | SymbolKind::UserDefinedSchema(_, _) => {
      TypeResult::new(db, Some(Box::new(TdrTypeType::get(db))), vec![])
    }
  }
}
