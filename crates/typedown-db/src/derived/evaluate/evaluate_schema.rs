//! Evaluate a schema symbol to extract its type

use typedown_macros::query_derived;

use crate::derived::get_builtin_types::{
  get_bool_type, get_date_type, get_datetime_type, get_dict_type, get_link_type, get_list_type,
  get_num_type, get_schema_type, get_str_type, get_time_type,
};
use crate::types::{BuiltinSchemaKind, Symbol, SymbolKind, TdrTypeLike, TypeResult};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn evaluate_schema(db: &TypedownDatabase, symbol: Symbol) -> TypeResult {
  match symbol.kind(db) {
    SymbolKind::BuiltinSchema(kind) => {
      let typ: Box<dyn TdrTypeLike> = match kind {
        BuiltinSchemaKind::Schema => Box::new(get_schema_type(db)),
        BuiltinSchemaKind::Str => Box::new(get_str_type(db)),
        BuiltinSchemaKind::Num => Box::new(get_num_type(db)),
        BuiltinSchemaKind::Bool => Box::new(get_bool_type(db)),
        BuiltinSchemaKind::Date => Box::new(get_date_type(db)),
        BuiltinSchemaKind::DateTime => Box::new(get_datetime_type(db)),
        BuiltinSchemaKind::Time => Box::new(get_time_type(db)),
        BuiltinSchemaKind::List => Box::new(get_list_type(db)),
        BuiltinSchemaKind::Dict => Box::new(get_dict_type(db)),
        BuiltinSchemaKind::Link => Box::new(get_link_type(db)),
      };
      TypeResult::new(db, typ, vec![])
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
