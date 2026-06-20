mod derived;
mod input;
mod interned;
mod inventory;
mod untracked;

use std::any::Any;

pub use derived::*;
pub use input::*;
pub use interned::*;
pub use inventory::*;
pub use untracked::*;

pub trait Ingredient: Any + Send + Sync {
  /// Returns true if the entry at `arg_id` is still valid compared to `last_changed_at`
  fn green_check(
    &self,
    db: &dyn crate::QueryDatabase,
    arg_id: usize,
    last_changed_at: usize,
  ) -> bool;

  /// Force re-execution of the entry at `arg_id`
  fn re_execute(&self, db: &dyn crate::QueryDatabase, arg_id: usize);
}
