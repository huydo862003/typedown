//! Derived queries for constructing builtin type singletons

use typedown_macros::query_derived;

use crate::{QueryDatabase, TypedownDatabase};
use crate::types::{
  TdrBoolType, TdrDateTimeType, TdrDateType, TdrEnumType, TdrFuncType, TdrListType, TdrNumType,
  TdrObjectType, TdrRecordType, TdrStrType, TdrTimeType,
};

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
pub fn get_enum_type(db: &TypedownDatabase) -> TdrEnumType {
  TdrEnumType::new(db)
}
