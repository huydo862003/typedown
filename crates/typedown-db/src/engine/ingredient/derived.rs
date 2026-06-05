use dashmap::DashMap;

use crate::DerivedId;

/// A dependency recorded during a derived query execution
#[derive(Clone)]
pub struct Dependency {
  pub ingredient_index: usize, // Which ingredient
  pub entry_id: usize,         // Which entry in that ingredient
  pub changed_at: usize,       // The revision it had when we read it
}

/// A memoized derived query result, mapping (key) to a derived struct ID
pub struct StampedDerivedQuery<V: DerivedId> {
  pub value: V,                      // The derived struct ID
  pub changed_at: usize,             // Revision when the value last actually changed
  pub verified_at: usize,            // Revision when last confirmed valid
  pub dependencies: Vec<Dependency>, // What this query read during execution
}

/// Ingredient for a derived query function: maps key tuple to memoized result
#[doc(hidden)]
pub struct DerivedQueryIngredient<K, V: DerivedId> {
  #[doc(hidden)]
  pub data: DashMap<K, StampedDerivedQuery<V>>,
}

impl<K: Eq + std::hash::Hash, V: crate::DerivedId> DerivedQueryIngredient<K, V> {
  pub fn new() -> Self {
    Self {
      data: DashMap::new(),
    }
  }
}

/// A stamped field value for a derived struct
pub struct StampedDerivedField<T> {
  pub value: T,
  pub changed_at: usize, // The last revision number this one changed
}

/// Ingredient for a derived struct field: maps entry id to stamped value
#[doc(hidden)]
pub struct DerivedFieldIngredient<T> {
  #[doc(hidden)]
  pub data: DashMap<usize, StampedDerivedField<T>>,
}

impl<T> DerivedFieldIngredient<T> {
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  pub const __TYPEDOWN_DERIVED_FIELD_INGREDIENT: () = ();

  pub fn new() -> Self {
    Self {
      data: DashMap::new(),
    }
  }
}
