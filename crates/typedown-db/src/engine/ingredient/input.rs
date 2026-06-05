// TIL: We use DashMap to support high-performance concrruent reads, which fits the workload of IDEs
use dashmap::DashMap;

/// A field of an input ingredient, containing data for that input type
#[doc(hidden)]
pub struct InputFieldIngredient<T> {
  // A map from id to field value
  // DashMap is used to better support parallel workload
  #[doc(hidden)]
  pub data: DashMap<usize, T>,
}

impl<T> InputFieldIngredient<T> {
  /// Marker used by the `query_db` macro to verify the input ingredient at compile time.
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  pub const __TYPEDOWN_INPUT_FIELD_INGREDIENT: () = ();

  pub fn new() -> Self {
    Self {
      data: DashMap::new(),
    }
  }
}
