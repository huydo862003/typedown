// TIL: We use DashMap to support high-performance concurrent reads, which fits the workload of IDEs
use dashmap::DashMap;

/// A dependency recorded during a derived query execution
#[derive(Clone)]
pub struct Dependency {
  pub ingredient_index: usize, // Which ingredient (input field or derived)
  pub entry_id: usize,         // Which entry in that ingredient
  pub changed_at: usize,       // The revision it had when we read it
}

pub struct StampedInputField<T> {
  pub value: T,
  pub changed_at: usize, // The last revision number this one changed
}

/// A memoized derived query result
pub struct StampedDerivedField<V> {
  pub value: V,                      // The cached result
  pub changed_at: usize,             // Revision when the value last actually changed
  pub verified_at: usize,            // Revision when last confirmed valid
  pub dependencies: Vec<Dependency>, // What this query read during execution
}

/// A generic ingredient backed by a DashMap
#[doc(hidden)]
pub struct Ingredient<K, V> {
  #[doc(hidden)]
  pub data: DashMap<K, V>,
}

impl<K: Eq + std::hash::Hash, V> Ingredient<K, V> {
  pub fn new() -> Self {
    Self {
      data: DashMap::new(),
    }
  }
}

/// An input field ingredient: maps entry id to stamped value
pub type InputFieldIngredient<T> = Ingredient<usize, StampedInputField<T>>;

/// A derived query ingredient: maps key tuple to stamped derived result
pub type DerivedIngredient<K, V> = Ingredient<K, StampedDerivedField<V>>;
