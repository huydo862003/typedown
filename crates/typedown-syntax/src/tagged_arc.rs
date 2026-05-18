//! A thread-safe read-only reference-counted pointer with a stolen last bit for some assertions about the value it points to

use std::{
  marker::PhantomData,
  sync::atomic::{AtomicUsize, Ordering, fence},
};

// A dynamically sized type (the last field)
// TIL: Only the last field can be dynamically sized (like C)
struct TaggedArcInner<T: ?Sized> {
  // An atomic ref count
  // Note: I suppose rust uses the same memory model as C++11 (acquire/release semantics)
  ref_count: AtomicUsize,
  data: T,
}

pub(crate) struct TaggedArc<L, R>(
  // A pointer to either TaggedArcInner<L> or TaggedArcInner<R>
  // The lowest bit is stolen:
  // - 0 -> left (L)
  // - 1 -> right (R)
  usize,
  // PhantomData<(L, R)> means TaggedArc<L, R> will implement Send and Sync as long as L and R
  // implement Send and Sync
  // TIL: We can make Rust auto infer Send + Sync inheritance using PhantomData, although it's not
  //      the main purpose here
  PhantomData<(L, R)>,
);

impl<L, R> TaggedArc<L, R> {
  /// Create a left TaggedArc
  // TIL: Use Box::into_raw is the canonical way to allocate on the heap and remove ownership
  pub(crate) fn new_left(left: L) -> Self {
    let inner = Box::new(TaggedArcInner {
      ref_count: AtomicUsize::new(1),
      data: left,
    });
    let ptr = Box::into_raw(inner) as usize;
    debug_assert!(ptr & 1 == 0, "[TaggedArc::new_left]: Pointer not aligned");
    Self(ptr, PhantomData)
  }

  /// Create a right TaggedArc
  // TIL: Use Box::into_raw is the canonical way to allocate on the heap and remove ownership
  pub(crate) fn new_right(right: R) -> Self {
    let inner = Box::new(TaggedArcInner {
      ref_count: AtomicUsize::new(1),
      data: right,
    });
    let ptr = Box::into_raw(inner) as usize;
    debug_assert!(ptr & 1 == 0, "[TaggedArc::new_right] pointer not aligned");
    Self(ptr | 1, PhantomData)
  }

  ///! Return whether the stored value is Left
  pub(crate) fn is_left(&self) -> bool {
    self.0 & 1 == 0
  }

  ///! Return whether the stored value is Right
  pub(crate) fn is_right(&self) -> bool {
    self.0 & 1 == 1
  }

  ///! Return the stored raw pointer cast as Left reference
  pub(crate) fn as_left(&self) -> Option<&L> {
    if !self.is_left() {
      return None;
    }
    let ptr = self.0 as *const TaggedArcInner<L>;
    Some(unsafe { &(*ptr).data })
  }

  ///! Return the stored raw pointer cast as Right reference
  pub(crate) fn as_right(&self) -> Option<&R> {
    if !self.is_right() {
      return None;
    }
    let ptr = (self.0 & !1) as *const TaggedArcInner<R>;
    Some(unsafe { &(*ptr).data })
  }
}

// TaggedArc<L, R> is Send + Sync if L and R are both Send + Sync
unsafe impl<L: Send + Sync, R: Send + Sync> Send for TaggedArc<L, R> {}
unsafe impl<L: Send + Sync, R: Send + Sync> Sync for TaggedArc<L, R> {}

impl<L, R> Clone for TaggedArc<L, R> {
  fn clone(&self) -> Self {
    let inner_ptr = (self.0 & !1) as *const TaggedArcInner<u8>;
    unsafe {
      (*inner_ptr)
        .ref_count
        // Relaxed suffices here: We just need
        // to publish to everyone that there's another owner here
        // No other established ordering is required
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    };
    Self(self.0, PhantomData)
  }
}

impl<L, R> Drop for TaggedArc<L, R> {
  fn drop(&mut self) {
    let inner_ptr = (self.0 & !1) as *const TaggedArcInner<u8>;
    let prev = unsafe { (*inner_ptr).ref_count.fetch_sub(1, Ordering::Release) }; // We need release here for the fence below
    if prev != 1 {
      return;
    }

    // Establish a synchronize-with ordering with all fetch_sub
    // This ensures that if this thread reads out 1, all other thread has moved past the `prev` point & all operations before that point, so data can safely be freed now
    fence(Ordering::Acquire);

    // Last reference: reconstruct the Box and let it drop
    unsafe {
      if self.is_left() {
        drop(Box::from_raw(self.0 as *mut TaggedArcInner<L>));
      } else {
        drop(Box::from_raw((self.0 & !1) as *mut TaggedArcInner<R>));
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tagged_arc_with_left_variant_basic_assertions() {
    let arc: TaggedArc<String, u32> = TaggedArc::new_left("hello".to_string());
    assert!(arc.is_left());
    assert!(!arc.is_right());
    assert_eq!(arc.as_left().unwrap(), "hello");
    assert!(arc.as_right().is_none());
  }

  #[test]
  fn tagged_arc_with_right_variant_basic_assertions() {
    let arc: TaggedArc<String, u32> = TaggedArc::new_right(42);
    assert!(arc.is_right());
    assert!(!arc.is_left());
    assert_eq!(*arc.as_right().unwrap(), 42);
    assert!(arc.as_left().is_none());
  }

  #[test]
  fn tagged_arc_clone_should_share_data() {
    let a: TaggedArc<String, u32> = TaggedArc::new_left("shared".to_string());
    let b = a.clone();
    assert_eq!(a.as_left().unwrap(), b.as_left().unwrap());
  }
}
