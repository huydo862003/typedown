//! GreenChild: A thread-safe readonly tagged pointer to either a GreenNode or a GreenToken

use std::hash::{Hash, Hasher};

use super::node::GreenNode;
use super::token::GreenToken;

/// GreenChild: A thread-safe readonly tagged pointer to either a GreenNode or a GreenToken
// Tag bit 0 = node, tag bit 1 = token
pub struct GreenChild(usize);

impl GreenChild {
  /// Take ownership of a GreenNode and store it as a tagged pointer.
  pub fn from_node(node: GreenNode) -> Self {
    let ptr = node.0 as usize;
    debug_assert!(ptr & 1 == 0, "GreenNode pointer not aligned");
    std::mem::forget(node); // Forgot the node to avoid it being dropped
    Self(ptr)
  }

  /// Take ownership of a GreenToken and store it as a tagged pointer.
  pub fn from_token(token: GreenToken) -> Self {
    let ptr = token.0 as usize;
    debug_assert!(ptr & 1 == 0, "GreenToken pointer not aligned");
    std::mem::forget(token); // Forgot the token to avoid it being dropped
    Self(ptr | 1)
  }

  /// Returns true if this child points to a GreenNode.
  pub fn is_node(&self) -> bool {
    self.0 & 1 == 0
  }

  /// Returns true if this child points to a GreenToken.
  pub fn is_token(&self) -> bool {
    self.0 & 1 == 1
  }

  /// Returns a new owned handle to the inner GreenNode (ref-count bumped via GreenNode::clone).
  /// Returns None if this child is a token.
  pub fn as_node(&self) -> Option<GreenNode> {
    if !self.is_node() {
      return None;
    }
    let tmp = GreenNode(self.0 as *const _);
    let cloned = tmp.clone();
    std::mem::forget(tmp);
    Some(cloned)
  }

  /// Returns a new owned handle to the inner GreenToken (ref-count bumped via GreenToken::clone).
  /// Returns None if this child is a node.
  pub fn as_token(&self) -> Option<GreenToken> {
    if !self.is_token() {
      return None;
    }
    let tmp = GreenToken((self.0 & !1) as *const _);
    let cloned = tmp.clone();
    std::mem::forget(tmp);
    Some(cloned)
  }

  pub fn text_len(&self) -> usize {
    if self.is_token() {
      self.as_token().unwrap().text_len()
    } else {
      self.as_node().unwrap().text_len()
    }
  }
}

impl Clone for GreenChild {
  fn clone(&self) -> Self {
    if self.is_node() {
      // Reconstruct a temporary handle,
      // clone it (bumps ref-count via GreenNode::clone),
      // forget the temporary (prevents decrement)
      let tmp = GreenNode(self.0 as *const _);
      let cloned = tmp.clone();
      std::mem::forget(tmp);
      Self::from_node(cloned)
    } else {
      let tmp = GreenToken((self.0 & !1) as *const _);
      let cloned = tmp.clone();
      std::mem::forget(tmp);
      Self::from_token(cloned)
    }
  }
}

impl Drop for GreenChild {
  fn drop(&mut self) {
    // Reconstruct the original handle and let its Drop handle
    unsafe {
      if self.is_node() {
        drop(GreenNode(self.0 as *const _));
      } else {
        drop(GreenToken((self.0 & !1) as *const _));
      }
    }
  }
}

impl PartialEq for GreenChild {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0 || {
      match (self.is_node(), other.is_node()) {
        (true, true) => {
          self.as_node().unwrap() == other.as_node().unwrap()
        }
        (false, false) => {
          self.as_token().unwrap() == other.as_token().unwrap()
        }
        _ => false,
      }
    }
  }
}

impl Eq for GreenChild {}

impl Hash for GreenChild {
  fn hash<H: Hasher>(&self, state: &mut H) {
    if self.is_node() {
      0u8.hash(state); // discriminant
      self.as_node().unwrap().hash(state);
    } else {
      1u8.hash(state); // discriminant
      self.as_token().unwrap().hash(state);
    }
  }
}

unsafe impl Send for GreenChild {}
unsafe impl Sync for GreenChild {}
