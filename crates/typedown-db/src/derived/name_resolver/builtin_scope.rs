use std::collections::HashMap;

use typedown_macros::query_derived;

use crate::derived::get_builtin_types::get_schema_symbol;
use crate::types::Symbol;
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub struct BuiltinScopeMembers {
  pub members: HashMap<String, Symbol>,
}

#[query_derived]
pub fn builtin_schema_scope(db: &TypedownDatabase) -> BuiltinScopeMembers {
  let mut members = HashMap::new();
  members.insert("Schema".to_string(), get_schema_symbol(db));
  BuiltinScopeMembers::new(db, members)
}

#[query_derived]
pub fn builtin_resource_scope(db: &TypedownDatabase) -> BuiltinScopeMembers {
  BuiltinScopeMembers::new(db, HashMap::new())
}
