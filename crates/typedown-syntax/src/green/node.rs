use std::alloc::{Layout, alloc, dealloc};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::child::GreenChild;
use crate::green::syntax_kind::SyntaxKind;

pub(super) struct GreenNodeHeader {
  pub(super) ref_count: AtomicUsize,
  pub(super) kind: SyntaxKind,
  pub(super) text_len: usize,
  pub(super) n_children: u32,
}

/// An interior node in the green tree.
pub struct GreenNode(pub(super) *const GreenNodeHeader);

impl GreenNode {
  fn layout(n: usize) -> (Layout, usize) {
    Layout::new::<GreenNodeHeader>()
      .extend(Layout::array::<GreenChild>(n).unwrap())
      .unwrap()
  }

  pub(super) fn from_raw_parts(kind: SyntaxKind, children: &[GreenChild]) -> Self {
    let n = children.len();
    let text_len = children.iter().map(|c| c.text_len()).sum();
    let (layout, children_offset) = Self::layout(n);

    unsafe {
      let base = alloc(layout);
      assert!(!base.is_null(), "allocation failed");

      (base as *mut GreenNodeHeader).write(GreenNodeHeader {
        ref_count: AtomicUsize::new(1),
        kind,
        text_len,
        n_children: n as u32,
      });

      // Clone each child (bumps ref-count) and write into the allocation.
      let children_ptr = base.add(children_offset) as *mut GreenChild;
      for (i, child) in children.iter().enumerate() {
        children_ptr.add(i).write(child.clone());
      }

      Self(base as *const GreenNodeHeader)
    }
  }

  /// Create a new interned GreenNode via the cache.
  pub fn new(cache: &mut super::cache::Cache, kind: SyntaxKind, children: &[GreenChild]) -> Self {
    cache.node(kind, children)
  }

  pub fn kind(&self) -> SyntaxKind {
    unsafe { (*self.0).kind }
  }

  pub fn text_len(&self) -> usize {
    unsafe { (*self.0).text_len }
  }

  pub fn n_children(&self) -> u32 {
    unsafe { (*self.0).n_children }
  }

  /// Returns a slice of this node's children.
  pub fn children(&self) -> &[GreenChild] {
    unsafe {
      let n = (*self.0).n_children as usize;
      let (_, offset) = Self::layout(n);
      let ptr = (self.0 as *const u8).add(offset) as *const GreenChild;
      std::slice::from_raw_parts(ptr, n)
    }
  }
}

impl Clone for GreenNode {
  fn clone(&self) -> Self {
    unsafe { (*self.0).ref_count.fetch_add(1, Ordering::AcqRel) };
    Self(self.0)
  }
}

impl Drop for GreenNode {
  fn drop(&mut self) {
    let prev = unsafe { (*self.0).ref_count.fetch_sub(1, Ordering::AcqRel) };
    if prev != 1 {
      return;
    }
    unsafe {
      let n = (*self.0).n_children as usize;
      let (layout, offset) = Self::layout(n);
      let children_ptr = (self.0 as *mut u8).add(offset) as *mut GreenChild;
      for i in 0..n {
        std::ptr::drop_in_place(children_ptr.add(i));
      }
      dealloc(self.0 as *mut u8, layout);
    }
  }
}

impl PartialEq for GreenNode {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0 || (self.kind() == other.kind() && self.children() == other.children())
  }
}

impl Eq for GreenNode {}

impl Hash for GreenNode {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.kind().hash(state);
    self.children().hash(state);
  }
}

unsafe impl Send for GreenNode {}
unsafe impl Sync for GreenNode {}
