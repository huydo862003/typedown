use std::sync::atomic::{AtomicUsize, Ordering};

// TIL: We use DashMap to support high-performance concrruent reads, which fits the workload of IDEs
use dashmap::DashMap;

/// A type of input, containing data for that input type
#[doc(hidden)]
pub struct InputIngredient<T> {
  next_id: AtomicUsize, // The next id to assign to a new input
  // A map from id to data tuples, converted from the original struct
  // DashMap is used to better support parallel workload
  #[doc(hidden)]
  pub data: DashMap<usize, T>,
}

impl<T> InputIngredient<T> {
  /// Marker used by the `query_db` macro to verify the input ingredient at compile time.
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  pub const __TYPEDOWN_INPUT_INGREDIENT: () = ();

  pub fn new() -> Self {
    Self {
      next_id: AtomicUsize::new(0),
      data: DashMap::new(),
    }
  }

  #[doc(hidden)]
  pub fn intern(&self, value: T) -> usize {
    let id = self.next_id.fetch_add(1, Ordering::Relaxed);
    self.data.insert(id, value);
    id
  }
}
