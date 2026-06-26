use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::sync::atomic::{AtomicBool, AtomicUsize};

use super::ingredient::{Dependency, Ingredient, IngredientFactory, Inventory};

/// A registry of ingredient factories
/// This is used in QueryStorage::default() to initialize the internal ingredient vector
/// TIL: By storing the callbacks (IngredientFactory is a function pointer type), instead of the empty dyn Ingredients (used as templates so default can clone), this avoid requiring the Ingredient to be cloneable... but Ingredient is a supertrait of Any, which is not clonable so cannot be used with dyn!
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
  pub disambiguator_map: HashMap<u64, usize>, // map hash(ingredient_index, id_field_values) to counter
}

#[derive(Clone)]
pub struct QueryStorage {
  #[doc(hidden)]
  pub revision: Arc<AtomicUsize>, // The current version of the query storage
  #[doc(hidden)]
  pub cancelled: Arc<AtomicBool>, // Set to true to cancel in-flight derived queries
  #[doc(hidden)]
  pub ingredients: Arc<Vec<Box<dyn Ingredient>>>, // All ingredients
}

impl QueryStorage {
  pub fn default() -> Self {
    QueryStorage {
      revision: Arc::new(AtomicUsize::new(0)),
      cancelled: Arc::new(AtomicBool::new(false)),
      ingredients: Arc::new(
        registry()
          .iter()
          .enumerate()
          .map(|(idx, factory)| factory(idx))
          .collect(),
      ),
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

  /// Get the next disambiguator for a given identity hash within the current query execution
  /// Returns 0 if not inside a query execution
  #[doc(hidden)]
  pub fn next_disambiguator(&self, identity_hash: u64) -> usize {
    self.with_context(|ctx| {
      if let Some(ctx) = ctx {
        let counter = ctx.disambiguator_map.entry(identity_hash).or_insert(0);
        let value = *counter;
        *counter += 1;
        value
      } else {
        0
      }
    })
  }

  /// Get the current query's identity (ingredient_index, arg_id) from the top of the query stack.
  /// Returns (0, 0) if not inside a query execution.
  #[doc(hidden)]
  pub fn current_query_identity(&self) -> (usize, usize) {
    self.with_context(|ctx| {
      if let Some(ctx) = ctx {
        ctx
          .query_stack
          .last()
          .map(|entry| (entry.ingredient_index, entry.arg_id))
          .unwrap_or((0, 0))
      } else {
        (0, 0)
      }
    })
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
