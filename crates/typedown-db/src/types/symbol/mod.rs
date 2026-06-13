use std::collections::HashMap;

use typedown_macros::query_derived;

use crate::types::TdrNode;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SymbolKind {
  Schema,
}

#[query_derived]
pub struct Symbol {
  #[id]
  node: TdrNode,
  kind: SymbolKind,
}

#[query_derived]
pub struct References {
  nodes: Vec<TdrNode>,
}

#[query_derived]
pub struct ProjectSchemaResult {
  members: HashMap<String, Symbol>,
}

#[query_derived]
pub struct MembersResult {
  schema_members: HashMap<String, Symbol>,
  resource_members: HashMap<String, Symbol>,
}
