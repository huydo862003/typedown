use std::sync::Arc;

use dashmap::DashMap;

use crate::{
  DeserializeContext, Fingerprint, QueryDatabase, SerializeContext, StableHash, StableHasher,
};

use super::Ingredient;

/// An ingredient for an interned struct
#[derive(Clone)]
#[doc(hidden)]
pub struct InternedIngredient<T> {
  ingredient_index: usize,
  name: &'static str,
  #[doc(hidden)]
  pub data: Arc<DashMap<usize, T>>,
}

impl<T> InternedIngredient<T> {
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  pub const __TYPEDOWN_INTERNED_INGREDIENT: () = ();

  pub fn new(ingredient_index: usize, name: &'static str) -> Self {
    Self {
      ingredient_index,
      name,
      data: Arc::new(DashMap::new()),
    }
  }
}

impl<T: StableHash + Send + Sync + 'static> InternedIngredient<T> {
  pub fn value_fingerprint(&self, db: &dyn QueryDatabase, arg_id: usize) -> Option<Fingerprint> {
    self.data.get(&arg_id).map(|entry| {
      let mut hasher = StableHasher::new();
      entry.value().stable_hash(db, &mut hasher);
      Fingerprint::from_hasher(hasher)
    })
  }
}

impl<T: Send + Sync + 'static> Ingredient for InternedIngredient<T> {
  fn name(&self) -> Fingerprint {
    Fingerprint::from_name(self.name)
  }

  fn green_check(&self, _db: &dyn QueryDatabase, _arg_id: usize, _last_changed_at: usize) -> bool {
    // Interned values never change, always green
    true
  }

  fn re_execute(&self, _db: &dyn QueryDatabase, _arg_id: usize) {
    // Interned values never change, nothing to recompute
  }

  fn serialize(&self, _ctx: &mut SerializeContext) {
    // TODO: implement serialization
  }

  fn deserialize(&self, _ctx: &mut DeserializeContext) {
    // TODO: implement deserialization
  }
}
