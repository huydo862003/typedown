//! The higher layer over green nodes
//! Which gives green node identity and the child nodes now contain a back pointers to their
//! parents

use std::{ops::Deref, rc::Rc};

use crate::green::{GreenNode, node::SyntaxNode};
use typedown_types::syntax_kind::SyntaxKind;

pub struct RedNodeData {
  /// The start offset of this red node in the source code
  offset: usize,
  /// The parent pointer
  parent: Option<RedNode>,
  /// The underlying green child
  green: GreenNode,
}

#[derive(Clone)]
pub struct RedNode(Rc<RedNodeData>);

impl RedNode {
  pub fn new_root(root: SyntaxNode) -> RedNode {
    RedNode(Rc::new(RedNodeData {
      offset: 0,
      parent: None,
      green: GreenNode::from_node(root),
    }))
  }

  pub fn kind(&self) -> SyntaxKind {
    self.0.green.kind()
  }

  pub fn parent(&self) -> Option<RedNode> {
    self.0.parent.clone()
  }

  pub fn children(&self) -> RedNodeChildren {
    let green_node = self.0.green.as_node();
    RedNodeChildren {
      parent: self.clone(),
      green_node: green_node.map(|n| n.clone()),
      index: 0,
      offset: self.0.offset,
    }
  }
}

impl Deref for RedNode {
  type Target = GreenNode;

  fn deref(&self) -> &Self::Target {
    &self.0.green
  }
}

/// A lazy iterator over a RedNode's children.
pub struct RedNodeChildren {
  parent: RedNode,
  green_node: Option<SyntaxNode>,
  index: usize,
  offset: usize,
}

impl Iterator for RedNodeChildren {
  type Item = RedNode;

  fn next(&mut self) -> Option<RedNode> {
    let children = self.green_node.as_ref()?.children();
    let child = children.get(self.index)?;
    let child_offset = self.offset;
    self.offset += child.text_len();
    self.index += 1;
    Some(RedNode(Rc::new(RedNodeData {
      offset: child_offset,
      parent: Some(self.parent.clone()),
      green: child.clone(),
    })))
  }
}
