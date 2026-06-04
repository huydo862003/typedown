use std::sync::OnceLock;
use std::{any::Any, sync::atomic::AtomicUsize};

use super::ingredient::{IngredientFactory, Inventory};

/// A registry of ingredient factories
/// This is used in QueryStorage::default() to initialize the internal ingredient vector
/// TIL: By storing the callbacks, instead of the empty ingredients (used as templates so default can clone), this avoid requiring the  ingredient to cloneable... but Any is not clonable so cannot be used with dyn!
static INGREDIENT_REGISTRY: OnceLock<Vec<IngredientFactory>> = OnceLock::new();

pub struct QueryStorage {
  #[doc(hidden)]
  pub revision: AtomicUsize, // The current version of the query storage
  #[doc(hidden)]
  pub inputs: Vec<Box<dyn Any + Send + Sync>>, // Input ingredients
}

impl QueryStorage {
  pub fn default() -> Self {
    QueryStorage {
      revision: AtomicUsize::new(0),
      inputs: registry().iter().map(|factory| factory()).collect(),
    }
  }

  /// Marker used by the `query_db` macro to verify the storage field type at compile time.
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  pub const __TYPEDOWN_QUERY_STORAGE: () = ();

}

fn registry() -> &'static Vec<IngredientFactory> {
  INGREDIENT_REGISTRY.get_or_init(|| {
    let mut factories = Vec::new();

    for entry in crate::inventory::iter::<Inventory> {
      (entry.register)(&mut factories);
    }

    factories
  })
}
