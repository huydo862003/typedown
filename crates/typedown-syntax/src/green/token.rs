//! Syntax Token: The leaf node in the AST
//! Use the same SyntaxKind as Syntax Node

use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::green::syntax_kind::SyntaxKind;

pub(super) struct TokenBody {
  pub(super) ref_count: AtomicUsize,
  pub(super) kind: SyntaxKind,
  pub(super) bytes: Vec<u8>,
}

/// The leaf node in the green tree.
pub struct Token(pub(super) *const TokenBody);

impl Token {
  pub(crate) fn new(cache: &mut super::cache::Cache, kind: SyntaxKind, text: &[u8]) -> Self {
    cache.token(kind, text)
  }

  pub(super) fn from_raw_parts(kind: SyntaxKind, bytes: Vec<u8>) -> Self {
    let body = Box::new(TokenBody {
      ref_count: AtomicUsize::new(1),
      kind,
      bytes,
    });
    Self(Box::into_raw(body))
  }

  pub fn kind(&self) -> SyntaxKind {
    unsafe { (*self.0).kind }
  }

  pub fn text(&self) -> impl Iterator<Item = char> {
    let bytes = unsafe { &(*self.0).bytes };
    bytes
      .iter()
      .map(|b| u32::from(*b))
      .map(|val| char::from_u32(val))
      .map(|maybe_char| maybe_char.unwrap_or('\u{FFFD}'))
  }

  pub fn bytes(&self) -> &[u8] {
    unsafe { &(*self.0).bytes }
  }

  pub fn text_len(&self) -> usize {
    unsafe { (*self.0).bytes.len() }
  }
}

impl Clone for Token {
  /// The clone is very cheap
  /// Suggest to use clone instead of &
  fn clone(&self) -> Self {
    // Currently use AcqRel for extra safety
    unsafe { (*self.0).ref_count.fetch_add(1, Ordering::AcqRel) };
    Self(self.0)
  }
}

impl Drop for Token {
  fn drop(&mut self) {
    // Currently use AcqRel for extra safety
    let prev = unsafe { (*self.0).ref_count.fetch_sub(1, Ordering::AcqRel) };
    if prev != 1 {
      return;
    }
    unsafe { drop(Box::from_raw(self.0 as *mut TokenBody)) };
  }
}

impl PartialEq for Token {
  fn eq(&self, other: &Self) -> bool {
    let self_bytes = unsafe { &(*self.0).bytes };
    let other_bytes = unsafe { &(*other.0).bytes };

    self.0 == other.0
      || self.kind() == other.kind()
        && self_bytes.len() == other_bytes.len()
        && self_bytes == other_bytes
  }
}

impl Eq for Token {}

impl Hash for Token {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.kind().hash(state);
    self.text().collect::<String>().hash(state);
  }
}

unsafe impl Send for Token {}
unsafe impl Sync for Token {}
