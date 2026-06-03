use std::any::Any;
#[cfg(debug_assertions)]
use std::sync::atomic::AtomicUsize;

// TIL: We use DashMap to support high-performance concrruent reads, which fits the workload of IDEs
use dashmap::DashMap;

/// A type of input, containing data for that input type
#[doc(hidden)]
pub struct InputIngredient<T> {
  #[doc(hidden)]
  pub data: DashMap<usize, T>, // A map from id to data tuples, converted from the original struct
}

impl<T> InputIngredient<T> {
  /// Marker used by the `query_db` macro to verify the input ingredient at compile time.
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  pub const __TYPEDOWN_INPUT_INGREDIENT: () = ();
}

pub struct QueryStorage {
  pub inputs: Vec<Box<dyn Any>>, // A vector whose entries are input ingredients
}

impl QueryStorage {
  /// Marker used by the `query_db` macro to verify the storage field type at compile time.
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  pub const __TYPEDOWN_QUERY_STORAGE: () = ();

  #[cfg(debug_assertions)]
  #[doc(hidden)]
  pub const INPUT_INDEX: AtomicUsize = AtomicUsize::new(0);
}
