//! Evaluate a schema symbol to extract its type

use typedown_macros::query_derived;

use crate::derived::get_builtin_types::get_schema_type;
use crate::types::{BuiltinSchemaKind, Symbol, SymbolKind, TypeResult};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn evaluate_schema(db: &TypedownDatabase, symbol: Symbol) -> TypeResult {
  match symbol.kind(db) {
    SymbolKind::BuiltinSchema(BuiltinSchemaKind::Schema) => {
      TypeResult::new(db, Box::new(get_schema_type(db)), vec![])
    }
    SymbolKind::UserDefinedSchema(_file) => {
      todo!()
    }
  }
}
