mod derived;
mod input;
mod interned;
mod inventory;

use std::any::Any;

pub use derived::*;
pub use input::*;
pub use interned::*;
pub use inventory::*;

use crate::persist::serialized::dep_graph::DepNodeIndex;
use crate::{DepId, DeserializeContext, Fingerprint, QueryDatabase, SerializeContext};

pub trait Ingredient: Any + Send + Sync {
  fn name(&self) -> Fingerprint;

  fn green_check(&self, db: &dyn QueryDatabase, arg_id: usize, last_changed_at: usize) -> bool;

  fn re_execute(&self, db: &dyn QueryDatabase, arg_id: usize);

  fn entry_ids(&self) -> Box<dyn Iterator<Item = usize> + '_>;

  fn value_fingerprint(&self, db: &dyn QueryDatabase, entry_id: usize) -> Option<Fingerprint>;

  fn serialize(&self, ctx: &mut SerializeContext, entry_id: usize);

  /// Load a dep node into this ingredient's storage.
  fn deserialize(&self, ctx: &DeserializeContext, node_index: DepNodeIndex) -> Option<DepId>;
}
