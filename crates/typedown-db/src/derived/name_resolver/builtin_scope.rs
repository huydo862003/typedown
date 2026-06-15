use std::collections::HashMap;

use typedown_macros::query_derived;

use crate::derived::get_builtin_types::{
  get_bool_symbol, get_date_symbol, get_datetime_symbol, get_dict_symbol, get_link_symbol,
  get_list_symbol, get_num_symbol, get_schema_symbol, get_str_symbol, get_time_symbol,
  get_type_type_symbol,
};
use crate::types::Symbol;
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub struct BuiltinScopeMembers {
  pub members: HashMap<String, Symbol>,
}

#[query_derived]
pub fn builtin_schema_scope(db: &TypedownDatabase) -> BuiltinScopeMembers {
  let members = HashMap::from([
    ("Schema".to_string(), get_schema_symbol(db)),
    ("string".to_string(), get_str_symbol(db)),
    ("number".to_string(), get_num_symbol(db)),
    ("boolean".to_string(), get_bool_symbol(db)),
    ("date".to_string(), get_date_symbol(db)),
    ("datetime".to_string(), get_datetime_symbol(db)),
    ("time".to_string(), get_time_symbol(db)),
    ("list".to_string(), get_list_symbol(db)),
    ("dict".to_string(), get_dict_symbol(db)),
    ("link".to_string(), get_link_symbol(db)),
    ("type".to_string(), get_type_type_symbol(db)),
  ]);
  BuiltinScopeMembers::new(db, members)
}

#[query_derived]
pub fn builtin_resource_scope(db: &TypedownDatabase) -> BuiltinScopeMembers {
  BuiltinScopeMembers::new(db, HashMap::new())
}
