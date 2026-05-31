// Inspired by rust-analyzer

pub mod cache;
pub mod node;
pub mod token;

use std::hash::{Hash, Hasher};
use typedown_types::either::Either;

pub use node::SyntaxNode;
pub use token::SyntaxToken;

/// GreenNode: A thread-safe readonly tagged pointer to either a Node or a Token.
// Tag bit 0 = node, tag bit 1 = token
pub struct GreenNode(usize);

impl GreenNode {
  /// Take ownership of a Node and store it as a tagged pointer.
  pub fn from_node(node: SyntaxNode) -> Self {
    let ptr = node.0 as usize;
    debug_assert!(ptr & 1 == 0, "Node pointer not aligned");
    std::mem::forget(node);
    Self(ptr)
  }

  /// Take ownership of a Token and store it as a tagged pointer.
  pub fn from_token(token: SyntaxToken) -> Self {
    let ptr = token.0 as usize;
    debug_assert!(ptr & 1 == 0, "Token pointer not aligned");
    std::mem::forget(token);
    Self(ptr | 1)
  }

  /// Returns true if this child points to a Node.
  pub fn is_node(&self) -> bool {
    self.0 & 1 == 0
  }

  /// Returns true if this child points to a Token.
  pub fn is_token(&self) -> bool {
    self.0 & 1 == 1
  }

  /// Returns a new borrowed handle to the inner Node.
  /// Returns None if this child is a token.
  pub fn as_node(&self) -> Option<&SyntaxNode> {
    if !self.is_node() {
      return None;
    }
    let ptr = self.0 as *const _;
    unsafe { Some(&*ptr) }
  }

  /// Returns a borrowed handle to the inner Token.
  /// Returns None if this child is a node.
  pub fn as_token(&self) -> Option<&SyntaxToken> {
    if !self.is_token() {
      return None;
    }
    let ptr = (self.0 & !1) as *const _;
    unsafe { Some(&*ptr) }
  }

  pub fn kind(&self) -> typedown_types::syntax_kind::SyntaxKind {
    if self.is_token() {
      self.as_token().unwrap().kind()
    } else {
      self.as_node().unwrap().kind()
    }
  }

  pub fn text_len(&self) -> usize {
    if self.is_token() {
      self.as_token().unwrap().text_len()
    } else {
      self.as_node().unwrap().text_len()
    }
  }

  // We need to use Box<dyn Iterator> here due to rust cannot resolve recursive opaque types
  // TIL: Rust iterator type is really opaque and hardly writable. It mirrors the operations you've performed on the iterator. I think this is used for optimization
  pub fn chars<'a>(
    &'a self,
  ) -> Either<impl Iterator<Item = char>, Box<dyn Iterator<Item = char> + 'a>> {
    if self.is_token() {
      Either::Left(self.as_token().unwrap().chars())
    } else {
      Either::Right(Box::new(self.as_node().unwrap().chars()))
    }
  }
}

impl Clone for GreenNode {
  fn clone(&self) -> Self {
    if self.is_node() {
      let tmp = SyntaxNode(self.0 as *const _);
      let cloned = tmp.clone();
      std::mem::forget(tmp);
      Self::from_node(cloned)
    } else {
      let tmp = SyntaxToken((self.0 & !1) as *const _);
      let cloned = tmp.clone();
      std::mem::forget(tmp);
      Self::from_token(cloned)
    }
  }
}

impl Drop for GreenNode {
  fn drop(&mut self) {
    if self.is_node() {
      drop(SyntaxNode(self.0 as *const _));
    } else {
      drop(SyntaxToken((self.0 & !1) as *const _));
    }
  }
}

impl PartialEq for GreenNode {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0 || {
      match (self.is_node(), other.is_node()) {
        (true, true) => self.as_node().unwrap() == other.as_node().unwrap(),
        (false, false) => self.as_token().unwrap() == other.as_token().unwrap(),
        _ => false,
      }
    }
  }
}

impl Eq for GreenNode {}

impl Hash for GreenNode {
  fn hash<H: Hasher>(&self, state: &mut H) {
    if self.is_node() {
      0u8.hash(state);
      self.as_node().unwrap().hash(state);
    } else {
      1u8.hash(state);
      self.as_token().unwrap().hash(state);
    }
  }
}

unsafe impl Send for GreenNode {}
unsafe impl Sync for GreenNode {}
