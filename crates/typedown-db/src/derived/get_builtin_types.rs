//! Derived queries for constructing builtin type singletons

use typedown_macros::query_derived;

use crate::types::{
  TdrBoolObj, TdrBoolType, TdrDateTimeType, TdrDateType, TdrFuncType, TdrListType, TdrNumType,
  TdrObjectType, TdrRecordType, TdrSchemaType, TdrStrType, TdrTimeType,
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
pub fn get_func_type(db: &TypedownDatabase) -> TdrFuncType {
  TdrFuncType::new(db)
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
