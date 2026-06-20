//! An untracked ingredient
//! Useful for fields that are not needed to be tracked
//! And lift the requirements that tracked fields must implement Clone + PartialEq + Hash

use dashmap::DashMap;

use crate::{Ingredient, QueryDatabase};

#[doc(hidden)]
pub struct UntrackedFieldIngredient<T> {
  #[doc(hidden)]
  pub data: DashMap<usize, T>,
}

impl<T> UntrackedFieldIngredient<T> {
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  pub const __TYPEDOWN_UNTRACKED_INGREDIENT: () = ();

  pub fn new() -> Self {
    Self { data: DashMap::new() }
  }
}

impl<T: Send + Sync + 'static> Ingredient for UntrackedFieldIngredient<T> {
  fn green_check(&self, _db: &dyn QueryDatabase, _arg_id: usize, _last_changed_at: usize) -> bool {
    // Untracked values never change, always green
    true
  }

  fn re_execute(&self, _db: &dyn QueryDatabase, _arg_id: usize) {
    // Untracked values are set by the queries, nothing to recompute
  }
}
