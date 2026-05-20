//! Syntax Token: The leaf node in the AST
//! Use the same SyntaxKind as Syntax Node

use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::green::syntax_kind::SyntaxKind;

pub(super) struct GreenTokenBody {
  pub(super) ref_count: AtomicUsize,
  pub(super) kind: SyntaxKind,
  pub(super) text: String,
}

/// The leaf node in the green tree.
pub struct GreenToken(pub(super) *const GreenTokenBody);

impl GreenToken {
  pub fn new(cache: &mut super::cache::Cache, kind: SyntaxKind, text: &str) -> Self {
    cache.token(kind, text)
  }

  pub(super) fn from_raw_parts(kind: SyntaxKind, text: String) -> Self {
    let body = Box::new(GreenTokenBody {
      ref_count: AtomicUsize::new(1),
      kind,
      text,
    });
    Self(Box::into_raw(body))
  }

  pub fn kind(&self) -> SyntaxKind {
    unsafe { (*self.0).kind }
  }

  pub fn text(&self) -> &str {
    unsafe { &(*self.0).text }
  }

  pub fn text_len(&self) -> usize {
    self.text().len()
  }
}

impl Clone for GreenToken {
  fn clone(&self) -> Self {
    // Currently use AcqRel for extra safety
    unsafe { (*self.0).ref_count.fetch_add(1, Ordering::AcqRel) };
    Self(self.0)
  }
}

impl Drop for GreenToken {
  fn drop(&mut self) {
    // Currently use AcqRel for extra safety
    let prev = unsafe { (*self.0).ref_count.fetch_sub(1, Ordering::AcqRel) };
    if prev != 1 {
      return;
    }
    unsafe { drop(Box::from_raw(self.0 as *mut GreenTokenBody)) };
  }
}

impl PartialEq for GreenToken {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0 || (self.kind() == other.kind() && self.text() == other.text())
  }
}

impl Eq for GreenToken {}

impl Hash for GreenToken {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.kind().hash(state);
    self.text().hash(state);
  }
}

unsafe impl Send for GreenToken {}
unsafe impl Sync for GreenToken {}
