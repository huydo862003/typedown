use typedown_macros::query_interned;
use typedown_syntax::{ast::AstNode, red::RedNode};

use crate::QueryDatabase;

#[query_interned]
pub struct TdrNode {
  node: RedNode,
}

impl TdrNode {
  pub fn try_cast<T: AstNode>(&self, db: &(impl QueryDatabase + ?Sized)) -> Option<T> {
    T::cast(self.node(db))
  }
}
