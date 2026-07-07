use std::any::Any;

use super::storage::QueryStorage;
use crate::{DeserializeContext, SerializeContext};

pub trait QueryDatabase: Any {
  #[doc(hidden)]
  unsafe fn storage(&self) -> &QueryStorage;

  #[doc(hidden)]
  unsafe fn storage_mut(&mut self) -> &mut QueryStorage;

  /// Serialize all ingredients into the given context.
  fn dump(&self, ctx: &mut dyn SerializeContext) {
    let storage = unsafe { self.storage() };
    for entry in storage.ingredients.iter() {
      entry.ingredient.serialize(ctx);
    }
  }

  /// Deserialize all ingredients from the given context.
  fn load(&mut self, ctx: &mut dyn DeserializeContext) {
    let storage = unsafe { self.storage() };
    for entry in storage.ingredients.iter() {
      entry.ingredient.deserialize(ctx);
    }
  }
}
