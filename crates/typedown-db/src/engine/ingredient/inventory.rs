use super::Ingredient;

/// A callback that creates an Ingredient, receiving its index in the ingredients vec
pub type IngredientFactory = fn(usize) -> Box<dyn Ingredient>;

pub enum IngredientKind {
  Input,
  Derived,
}

pub struct Inventory {
  pub kind: IngredientKind,
  pub register: fn(&mut Vec<IngredientFactory>),
}

crate::inventory::collect!(Inventory);
