use std::collections::HashMap;

use typedown_macros::query_derived;

use crate::types::GreenNode;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SymbolKind {
  Schema,
}

#[query_derived]
pub struct Symbol {
  #[id]
  node: GreenNode,
  kind: SymbolKind,
}

#[query_derived]
pub struct References {
  nodes: Vec<GreenNode>,
}

#[query_derived]
pub struct MembersResult {
  schema_members: HashMap<String, Symbol>,
  resource_members: HashMap<String, Symbol>,
}
