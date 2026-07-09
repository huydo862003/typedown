mod derived;
mod input;
mod interned;
mod inventory;

use std::any::Any;

pub use derived::*;
pub use input::*;
pub use interned::*;
pub use inventory::*;

use crate::{Fingerprint, QueryDatabase, SerializeContext};

pub trait Ingredient: Any + Send + Sync {
  /// Stable fingerprint name for this ingredient, used for cross-session dep graph serialization.
  fn name(&self) -> Fingerprint;

  /// Returns true if the entry at `arg_id` is still valid compared to `last_changed_at`
  fn green_check(&self, db: &dyn QueryDatabase, arg_id: usize, last_changed_at: usize) -> bool;

  /// Force re-execution of the entry at `arg_id`
  fn re_execute(&self, db: &dyn crate::QueryDatabase, arg_id: usize);

  /// Returns all entry ids currently stored in this ingredient.
  fn entry_ids(&self) -> Box<dyn Iterator<Item = usize> + '_>;

  /// Compute the value fingerprint for an entry, used for cross-session green checks.
  fn value_fingerprint(&self, db: &dyn QueryDatabase, entry_id: usize) -> Option<Fingerprint>;

  fn serialize(&self, ctx: &mut SerializeContext, entry_id: usize);
}
