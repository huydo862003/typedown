use typedown_macros::query_interned;
use typedown_syntax::{ast::AstNode, red::RedNode};

use crate::QueryDatabase;

#[query_interned]
pub struct GreenNode {
  node: typedown_syntax::green::GreenNode,
}

impl GreenNode {
  // TIL: Before this, I thought try_cast<T: AstNode, DB: QueryDatabase + ?Sized> would always be
  // the same, except for this case
  pub fn try_cast<T: AstNode>(&self, db: &(impl QueryDatabase + ?Sized)) -> Option<T> {
    let node = self.node(db);
    let syntax = node.as_node()?;
    let red = RedNode::new_root(syntax.clone());
    T::cast(red)
  }
}
