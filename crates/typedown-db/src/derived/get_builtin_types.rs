//! Derived queries for constructing builtin type singletons

use typedown_macros::query_derived;

use typedown_types::diagnostic::Diagnostic;

use crate::types::FuncSignature;
use crate::types::{
  InstResult, Symbol, SymbolKind, TdrBoolObj, TdrBoolType, TdrDateTimeType, TdrDateType,
  TdrDictType, TdrFuncType, TdrListType, TdrMathType, TdrNumType, TdrObjectType,
  TdrSchemaPropertyType, TdrSchemaType, TdrStrType, TdrTimeType, TdrTypeLike, TdrTypeType,
};
use crate::{QueryDatabase, TypedownDatabase, types::BuiltinSchemaKind};

#[query_derived]
pub fn get_type_type(db: &TypedownDatabase) -> TdrTypeType {
  TdrTypeType::new(db)
}

#[query_derived]
pub fn get_object_type(db: &TypedownDatabase) -> TdrObjectType {
  TdrObjectType::new(db)
}

#[query_derived]
pub fn get_bool_type(db: &TypedownDatabase) -> TdrBoolType {
  TdrBoolType::new(db)
}

#[query_derived]
pub fn get_str_type(db: &TypedownDatabase) -> TdrStrType {
  TdrStrType::new(db)
}

#[query_derived]
pub fn get_num_type(db: &TypedownDatabase) -> TdrNumType {
  TdrNumType::new(db)
}

#[query_derived]
pub fn get_list_type(db: &TypedownDatabase) -> TdrListType {
  TdrListType::new(db, None)
}

#[query_derived]
pub fn get_dict_type(db: &TypedownDatabase) -> TdrDictType {
  TdrDictType::new(db, None, None)
}

#[query_derived]
pub fn get_math_type(db: &TypedownDatabase) -> TdrMathType {
  TdrMathType::new(db)
}

#[query_derived]
pub fn get_datetime_type(db: &TypedownDatabase) -> TdrDateTimeType {
  TdrDateTimeType::new(db)
}

#[query_derived]
pub fn get_date_type(db: &TypedownDatabase) -> TdrDateType {
  TdrDateType::new(db)
}

#[query_derived]
pub fn get_time_type(db: &TypedownDatabase) -> TdrTimeType {
  TdrTimeType::new(db)
}

#[query_derived]
pub fn get_true(db: &TypedownDatabase) -> TdrBoolObj {
  TdrBoolObj::new(db, true)
}

#[query_derived]
pub fn get_false(db: &TypedownDatabase) -> TdrBoolObj {
  TdrBoolObj::new(db, false)
}

#[query_derived]
pub fn get_schema_property_type(db: &TypedownDatabase) -> TdrSchemaPropertyType {
  TdrSchemaPropertyType::new(db)
}

// Schema type is actually a kind
// and its a subtype of the "type" kind
#[query_derived]
pub fn get_schema_type(db: &TypedownDatabase) -> TdrSchemaType {
  TdrSchemaType::new(db)
}

#[query_derived]
pub fn get_type_type_symbol(db: &TypedownDatabase) -> Symbol {
  Symbol::new(
    db,
    SymbolKind::BuiltinSchema(BuiltinSchemaKind::TypeType),
    "type".to_string(),
  )
}

#[query_derived]
pub fn get_schema_symbol(db: &TypedownDatabase) -> Symbol {
  Symbol::new(
    db,
    SymbolKind::BuiltinSchema(BuiltinSchemaKind::Schema),
    "schema".to_string(),
  )
}

#[query_derived]
pub fn get_str_symbol(db: &TypedownDatabase) -> Symbol {
  Symbol::new(
    db,
    SymbolKind::BuiltinSchema(BuiltinSchemaKind::Str),
    "string".to_string(),
  )
}

#[query_derived]
pub fn get_num_symbol(db: &TypedownDatabase) -> Symbol {
  Symbol::new(
    db,
    SymbolKind::BuiltinSchema(BuiltinSchemaKind::Num),
    "number".to_string(),
  )
}

#[query_derived]
pub fn get_bool_symbol(db: &TypedownDatabase) -> Symbol {
  Symbol::new(
    db,
    SymbolKind::BuiltinSchema(BuiltinSchemaKind::Bool),
    "boolean".to_string(),
  )
}

#[query_derived]
pub fn get_date_symbol(db: &TypedownDatabase) -> Symbol {
  Symbol::new(
    db,
    SymbolKind::BuiltinSchema(BuiltinSchemaKind::Date),
    "date".to_string(),
  )
}

#[query_derived]
pub fn get_datetime_symbol(db: &TypedownDatabase) -> Symbol {
  Symbol::new(
    db,
    SymbolKind::BuiltinSchema(BuiltinSchemaKind::DateTime),
    "datetime".to_string(),
  )
}

#[query_derived]
pub fn get_time_symbol(db: &TypedownDatabase) -> Symbol {
  Symbol::new(
    db,
    SymbolKind::BuiltinSchema(BuiltinSchemaKind::Time),
    "time".to_string(),
  )
}

#[query_derived]
pub fn get_math_symbol(db: &TypedownDatabase) -> Symbol {
  Symbol::new(
    db,
    SymbolKind::BuiltinSchema(BuiltinSchemaKind::Math),
    "math".to_string(),
  )
}

#[query_derived]
pub fn get_list_symbol(db: &TypedownDatabase) -> Symbol {
  Symbol::new(
    db,
    SymbolKind::BuiltinSchema(BuiltinSchemaKind::List),
    "list".to_string(),
  )
}

#[query_derived]
pub fn get_dict_symbol(db: &TypedownDatabase) -> Symbol {
  Symbol::new(
    db,
    SymbolKind::BuiltinSchema(BuiltinSchemaKind::Dict),
    "dict".to_string(),
  )
}

#[query_derived]
pub fn get_func_type(db: &TypedownDatabase, signature: FuncSignature) -> TdrFuncType {
  TdrFuncType::new(db, signature)
}

#[query_derived]
pub fn instantiate_type(
  db: &TypedownDatabase,
  constructor: Box<dyn TdrTypeLike>,
  args: Vec<Box<dyn TdrTypeLike>>,
) -> InstResult {
  let arity = constructor.arity(db);
  if arity != args.len() {
    return InstResult::new(
      db,
      dyn_clone::clone_box(&*constructor),
      vec![Diagnostic::WrongTypeArgCount {
        expected: arity,
        got: args.len(),
      }],
    );
  }
  constructor.instantiate(db, args)
}

#[cfg(test)]
mod tests {
  use typedown_types::diagnostic::Diagnostic;

  use crate::{
    QueryStorage, TypedownDatabase,
    derived::get_builtin_types::{
      get_dict_type, get_list_type, get_num_type, get_str_type, instantiate_type,
    },
    types::TdrTypeLike,
  };

  fn make_db() -> TypedownDatabase {
    TypedownDatabase {
      storage: QueryStorage::default(),
    }
  }

  #[test]
  fn instantiate_list_with_correct_arity() {
    let db = make_db();
    let list = Box::new(get_list_type(&db)) as Box<dyn TdrTypeLike>;
    let str_type = Box::new(get_str_type(&db)) as Box<dyn TdrTypeLike>;

    let result = instantiate_type(&db, list, vec![str_type.clone()]);

    assert!(
      result.diagnostics(&db).is_empty(),
      "expected no diagnostics"
    );
    let _expected = Box::new(get_str_type(&db)) as Box<dyn TdrTypeLike>;
    let instantiated = result.typ(&db);
    // The result should be a TdrListType with elem = str
    assert!(
      instantiated.arity(&db) == 0,
      "instantiated list should have arity 0"
    );
  }

  #[test]
  fn instantiate_record_with_correct_arity() {
    let db = make_db();
    let record = Box::new(get_dict_type(&db)) as Box<dyn TdrTypeLike>;
    let str_type = Box::new(get_str_type(&db)) as Box<dyn TdrTypeLike>;
    let num_type = Box::new(get_num_type(&db)) as Box<dyn TdrTypeLike>;

    let result = instantiate_type(&db, record, vec![str_type, num_type]);

    assert!(
      result.diagnostics(&db).is_empty(),
      "expected no diagnostics"
    );
    assert!(
      result.typ(&db).arity(&db) == 0,
      "instantiated record should have arity 0"
    );
  }

  #[test]
  fn instantiate_list_wrong_arity_produces_diagnostic() {
    let db = make_db();
    let list = Box::new(get_list_type(&db)) as Box<dyn TdrTypeLike>;

    let result = instantiate_type(&db, list, vec![]);

    let diagnostics = result.diagnostics(&db);
    assert_eq!(diagnostics.len(), 1);
    assert!(
      matches!(
        diagnostics[0],
        Diagnostic::WrongTypeArgCount {
          expected: 1,
          got: 0
        }
      ),
      "expected WrongTypeArgCount diagnostic"
    );
  }

  #[test]
  fn instantiate_record_wrong_arity_produces_diagnostic() {
    let db = make_db();
    let record = Box::new(get_dict_type(&db)) as Box<dyn TdrTypeLike>;
    let str_type = Box::new(get_str_type(&db)) as Box<dyn TdrTypeLike>;

    // Only 1 arg, record needs 2
    let result = instantiate_type(&db, record, vec![str_type]);

    let diagnostics = result.diagnostics(&db);
    assert_eq!(diagnostics.len(), 1);
    assert!(
      matches!(
        diagnostics[0],
        Diagnostic::WrongTypeArgCount {
          expected: 2,
          got: 1
        }
      ),
      "expected WrongTypeArgCount diagnostic"
    );
  }

  #[test]
  fn instantiate_arity0_type_with_no_args() {
    let db = make_db();
    let str_type = Box::new(get_str_type(&db)) as Box<dyn TdrTypeLike>;
    let expected = str_type.clone();

    let result = instantiate_type(&db, str_type, vec![]);

    assert!(
      result.diagnostics(&db).is_empty(),
      "expected no diagnostics"
    );
    assert!(
      result.typ(&db) == expected,
      "arity-0 type instantiated with no args should return itself"
    );
  }

  #[test]
  fn instantiate_arity0_type_with_extra_args_produces_diagnostic() {
    let db = make_db();
    let str_type = Box::new(get_str_type(&db)) as Box<dyn TdrTypeLike>;
    let num_type = Box::new(get_num_type(&db)) as Box<dyn TdrTypeLike>;

    let result = instantiate_type(&db, str_type, vec![num_type]);

    let diagnostics = result.diagnostics(&db);
    assert_eq!(diagnostics.len(), 1);
    assert!(
      matches!(
        diagnostics[0],
        Diagnostic::WrongTypeArgCount {
          expected: 0,
          got: 1
        }
      ),
      "expected WrongTypeArgCount diagnostic"
    );
  }
}
