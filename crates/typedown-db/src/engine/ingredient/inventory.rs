use super::Ingredient;

/// A callback that creates an Ingredient
pub type IngredientFactory = fn() -> Box<dyn Ingredient>;

pub enum IngredientKind {
  Input,
  Derived,
}

pub struct Inventory {
  pub kind: IngredientKind,
  pub register: fn(&mut Vec<IngredientFactory>),
}

crate::inventory::collect!(Inventory);
