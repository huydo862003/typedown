// TIL: We use DashMap to support high-performance concrruent reads, which fits the workload of IDEs
use std::sync::Arc;

use dashmap::DashMap;

use crate::QueryDatabase;

use super::Ingredient;

pub struct StampedInputField<T> {
  pub value: T,
  pub changed_at: usize, // The last revision number this one changed
}

/// A field of an input ingredient, containing data for that input type
#[derive(Clone)]
#[doc(hidden)]
pub struct InputFieldIngredient<T> {
  name: &'static str,
  // A map from id to field value
  // DashMap is used to better support parallel workload
  #[doc(hidden)]
  pub data: Arc<DashMap<usize, StampedInputField<T>>>,
}

impl<T> InputFieldIngredient<T> {
  /// Marker used by the `query_db` macro to verify the input ingredient at compile time.
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  pub const __TYPEDOWN_INPUT_FIELD_INGREDIENT: () = ();

  pub fn new(name: &'static str) -> Self {
    Self {
      name,
      data: Arc::new(DashMap::new()),
    }
  }
}

impl<T: Send + Sync + 'static> Ingredient for InputFieldIngredient<T> {
  fn name(&self) -> &'static str {
    self.name
  }

  fn green_check(&self, _db: &dyn QueryDatabase, arg_id: usize, last_changed_at: usize) -> bool {
    self
      .data
      .get(&arg_id)
      .map(|entry| entry.changed_at <= last_changed_at)
      .unwrap_or(false)
  }

  fn re_execute(&self, _db: &dyn QueryDatabase, _arg_id: usize) {
    // Inputs are ground truth, nothing to recompute
  }
}
