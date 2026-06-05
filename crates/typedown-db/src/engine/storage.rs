use std::cell::RefCell;
use std::sync::OnceLock;
use std::sync::atomic::AtomicUsize;

use super::ingredient::{Dependency, Ingredient, IngredientFactory, Inventory};

/// A registry of ingredient factories
/// This is used in QueryStorage::default() to initialize the internal ingredient vector
/// TIL: By storing the callbacks, instead of the empty ingredients (used as templates so default can clone), this avoid requiring the  ingredient to cloneable... but Any is not clonable so cannot be used with dyn!
static INGREDIENT_REGISTRY: OnceLock<Vec<IngredientFactory>> = OnceLock::new();

/// An entry in the query stack, used for cycle detection
pub struct QueryStackEntry {
  pub ingredient_index: usize,
  pub arg_id: usize,
}

/// Context passed through derived query execution
pub struct ExecuteContext {
  pub query_stack: Vec<QueryStackEntry>,
  pub dependencies: Vec<Dependency>,
}

pub struct QueryStorage {
  #[doc(hidden)]
  pub revision: AtomicUsize, // The current version of the query storage
  #[doc(hidden)]
  pub ingredients: Vec<Box<dyn Ingredient>>, // All ingredients (input fields and derived)
}

impl QueryStorage {
  pub fn default() -> Self {
    QueryStorage {
      revision: AtomicUsize::new(0),
      ingredients: registry().iter().map(|factory| factory()).collect(),
    }
  }

  /// Marker used by the `query_db` macro to verify the storage field type at compile time.
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  pub const __TYPEDOWN_QUERY_STORAGE: () = ();

  /// Access the current thread's ExecuteContext
  #[doc(hidden)]
  pub fn with_context<R>(&self, f: impl FnOnce(&mut Option<ExecuteContext>) -> R) -> R {
    thread_local! {
      static CTX: RefCell<Option<ExecuteContext>> = RefCell::new(None);
    }
    CTX.with(|c| f(&mut c.borrow_mut()))
  }
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
