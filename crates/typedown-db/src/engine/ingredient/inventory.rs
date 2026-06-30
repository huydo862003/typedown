use super::Ingredient;

/// An ingredient with its dep graph metadata
pub struct IngredientEntry {
  pub ingredient: Box<dyn Ingredient>,
  /// Field index within the parent struct, None for queries
  pub field_index: Option<u8>,
}

/// A callback that creates an IngredientEntry, receiving its index in the ingredients vec
pub type IngredientFactory = fn(usize) -> IngredientEntry;

pub struct Inventory {
  pub register: fn(&mut Vec<IngredientFactory>),
}

crate::inventory::collect!(Inventory);
