use std::sync::Arc;

use dashmap::DashMap;

use crate::QueryDatabase;

use super::Ingredient;

/// An ingredient for an interned struct
#[derive(Clone)]
#[doc(hidden)]
pub struct InternedIngredient<T> {
  #[doc(hidden)]
  pub data: Arc<DashMap<usize, T>>,
}

impl<T> InternedIngredient<T> {
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  pub const __TYPEDOWN_INTERNED_INGREDIENT: () = ();

  pub fn new() -> Self {
    Self {
      data: Arc::new(DashMap::new()),
    }
  }
}

impl<T: Send + Sync + 'static> Ingredient for InternedIngredient<T> {
  fn green_check(&self, _db: &dyn QueryDatabase, _arg_id: usize, _last_changed_at: usize) -> bool {
    // Interned values never change, always green
    true
  }

  fn re_execute(&self, _db: &dyn QueryDatabase, _arg_id: usize) {
    // Interned values never change, nothing to recompute
  }
}
