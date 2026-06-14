//! Derived queries for constructing builtin type singletons

use typedown_macros::query_derived;

use crate::types::FuncSignature;
use crate::types::{
  Symbol, SymbolKind, TdrBoolObj, TdrBoolType, TdrDateTimeType, TdrDateType, TdrFuncType,
  TdrListType, TdrNumType, TdrObjectType, TdrRecordType, TdrSchemaType, TdrStrType, TdrTimeType,
};
use crate::{QueryDatabase, TypedownDatabase};

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
  TdrListType::new(db)
}

#[query_derived]
pub fn get_record_type(db: &TypedownDatabase) -> TdrRecordType {
  TdrRecordType::new(db)
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
pub fn get_schema_type(db: &TypedownDatabase) -> TdrSchemaType {
  TdrSchemaType::new(db)
}

#[query_derived]
pub fn get_schema_symbol(db: &TypedownDatabase) -> Symbol {
  Symbol::new(
    db,
    SymbolKind::BuiltinSchema(crate::types::BuiltinSchemaKind::Schema),
  )
}

#[query_derived]
pub fn get_str_symbol(db: &TypedownDatabase) -> Symbol {
  Symbol::new(
    db,
    SymbolKind::BuiltinSchema(crate::types::BuiltinSchemaKind::Str),
  )
}

#[query_derived]
pub fn get_num_symbol(db: &TypedownDatabase) -> Symbol {
  Symbol::new(
    db,
    SymbolKind::BuiltinSchema(crate::types::BuiltinSchemaKind::Num),
  )
}

#[query_derived]
pub fn get_bool_symbol(db: &TypedownDatabase) -> Symbol {
  Symbol::new(
    db,
    SymbolKind::BuiltinSchema(crate::types::BuiltinSchemaKind::Bool),
  )
}

#[query_derived]
pub fn get_date_symbol(db: &TypedownDatabase) -> Symbol {
  Symbol::new(
    db,
    SymbolKind::BuiltinSchema(crate::types::BuiltinSchemaKind::Date),
  )
}

#[query_derived]
pub fn get_datetime_symbol(db: &TypedownDatabase) -> Symbol {
  Symbol::new(
    db,
    SymbolKind::BuiltinSchema(crate::types::BuiltinSchemaKind::DateTime),
  )
}

#[query_derived]
pub fn get_time_symbol(db: &TypedownDatabase) -> Symbol {
  Symbol::new(
    db,
    SymbolKind::BuiltinSchema(crate::types::BuiltinSchemaKind::Time),
  )
}

#[query_derived]
pub fn get_list_symbol(db: &TypedownDatabase) -> Symbol {
  Symbol::new(
    db,
    SymbolKind::BuiltinSchema(crate::types::BuiltinSchemaKind::List),
  )
}

#[query_derived]
pub fn get_record_symbol(db: &TypedownDatabase) -> Symbol {
  Symbol::new(
    db,
    SymbolKind::BuiltinSchema(crate::types::BuiltinSchemaKind::Record),
  )
}

#[query_derived]
pub fn get_func_type(db: &TypedownDatabase, signature: FuncSignature) -> TdrFuncType {
  TdrFuncType::new(db, signature)
}
