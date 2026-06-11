use typedown_macros::query_derived;

use crate::types::GreenNode;

#[query_derived]
pub struct Symbol {
  #[id]
  node: GreenNode,
}

#[query_derived]
pub struct References {
  nodes: Vec<GreenNode>,
}
