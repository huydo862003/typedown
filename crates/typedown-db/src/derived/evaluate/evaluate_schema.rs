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

#[cfg(test)]
mod tests {
  use crate::{
    QueryStorage, TypedownDatabase,
    derived::evaluate::evaluate_schema::evaluate_schema,
    derived::get_builtin_types::get_schema_type,
    types::{BuiltinSchemaKind, Symbol, SymbolKind, TdrTypeLike},
  };

  fn make_db() -> TypedownDatabase {
    TypedownDatabase {
      storage: QueryStorage::default(),
    }
  }

  #[test]
  fn evaluate_schema_builtin_schema_returns_schema_type() {
    let db = make_db();
    let symbol = Symbol::new(&db, SymbolKind::BuiltinSchema(BuiltinSchemaKind::Schema));

    let result = evaluate_schema(&db, symbol);

    let expected = Box::new(get_schema_type(&db)) as Box<dyn TdrTypeLike>;
    assert!(
      result.typ(&db) == expected,
      "builtin Schema symbol should evaluate to TdrSchemaType"
    );
    assert!(
      result.diagnostics(&db).is_empty(),
      "expected no diagnostics"
    );
  }
}
