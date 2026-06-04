use std::any::Any;
use std::sync::atomic::{AtomicUsize, Ordering};

// TIL: We use DashMap to support high-performance concrruent reads, which fits the workload of IDEs
use dashmap::DashMap;

/// A type of input, containing data for that input type
#[doc(hidden)]
pub struct InputIngredient<T> {
  next_id: AtomicUsize, // The next id to assign to a new input
  #[doc(hidden)]
  pub data: DashMap<usize, T>, // A map from id to data tuples, converted from the original struct
}

impl<T> InputIngredient<T> {
  /// Marker used by the `query_db` macro to verify the input ingredient at compile time.
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  pub const __TYPEDOWN_INPUT_INGREDIENT: () = ();

  #[doc(hidden)]
  pub fn intern(&self, value: T) -> usize {
    let id = self.next_id.fetch_add(1, Ordering::Relaxed);
    self.data.insert(id, value);
    id
  }
}

pub struct QueryStorage {
  inputs: DashMap<usize, Box<dyn Any + Send + Sync>>, // A map from ingredient index to input ingredients
}

impl QueryStorage {
  pub fn default() -> Self {
    QueryStorage {
      inputs: DashMap::default(),
    }
  }
  /// Marker used by the `query_db` macro to verify the storage field type at compile time.
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  pub const __TYPEDOWN_QUERY_STORAGE: () = ();

  #[cfg(debug_assertions)]
  #[doc(hidden)]
  pub const INPUT_INDEX: AtomicUsize = AtomicUsize::new(0);

  #[doc(hidden)]
  pub fn get_or_create_input_ingredient<T: Send + Sync + 'static>(
    &self,
    index: usize,
  ) -> dashmap::mapref::one::Ref<'_, usize, Box<dyn Any + Send + Sync>> {
    self.inputs.entry(index).or_insert_with(|| {
      Box::new(InputIngredient::<T> {
        next_id: AtomicUsize::new(0),
        data: DashMap::new(),
      })
    });
    self.inputs.get(&index).unwrap()
  }
}
