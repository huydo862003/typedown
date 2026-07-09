// TIL: We use DashMap to support high-performance concrruent reads, which fits the workload of IDEs
use std::sync::Arc;

use dashmap::DashMap;

use crate::{
  DeserializeContext, Encodable, Fingerprint, QueryDatabase, SerializeContext, StableHash,
  StableHasher, UnresolvedDepNode,
};

use super::Ingredient;

pub struct StampedInputField<T> {
  pub value: T,
  pub changed_at: usize, // The last revision number this one changed
}

/// A field of an input ingredient, containing data for that input type
#[derive(Clone)]
#[doc(hidden)]
pub struct InputFieldIngredient<T> {
  ingredient_index: usize,
  field_index: u8,
  name: &'static str,
  // A map from id to field value
  // DashMap is used to better support parallel workload
  #[doc(hidden)]
  pub data: Arc<DashMap<usize, StampedInputField<T>>>,
}

impl<T> InputFieldIngredient<T> {
  /// Marker used by the `query_db` macro to verify the input ingredient at compile time.
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  pub const __TYPEDOWN_INPUT_FIELD_INGREDIENT: () = ();

  pub fn new(ingredient_index: usize, name: &'static str, field_index: u8) -> Self {
    Self {
      ingredient_index,
      field_index,
      name,
      data: Arc::new(DashMap::new()),
    }
  }
}

impl<T: StableHash + Send + Sync + Encodable + 'static> Ingredient for InputFieldIngredient<T> {
  fn name(&self) -> Fingerprint {
    Fingerprint::from_name(self.name)
  }

  fn green_check(&self, _db: &dyn QueryDatabase, arg_id: usize, last_changed_at: usize) -> bool {
    self
      .data
      .get(&arg_id)
      .map(|entry| entry.changed_at <= last_changed_at)
      .unwrap_or(false)
  }

  fn re_execute(&self, _db: &dyn QueryDatabase, _arg_id: usize) {
    // Inputs are ground truth, nothing to recompute
  }

  fn entry_ids(&self) -> Box<dyn Iterator<Item = usize> + '_> {
    Box::new(self.data.iter().map(|entry| *entry.key()))
  }

  fn serialize(&self, ctx: &mut SerializeContext, entry_id: usize) {
    let entry = self.data.get(&entry_id);
    if entry.is_none() {
      return;
    }

    let entry = entry.expect("Entry must contain a value after the none check pass");

    // Add the dep node
    let dep_id = (self.ingredient_index, entry_id);
    let node_index = ctx.encoder.add_dep_id(dep_id);
    ctx.dep_graph.set(
      node_index,
      UnresolvedDepNode::InputField {
        name: self.name(),
        field_index: self.field_index,
        value: self
          .value_fingerprint(ctx.db(), entry_id)
          .expect("Entry is available so there must be a fingerprint"),
        changed_at: entry.changed_at as u64,
      },
    );

    // Encode and write to query cache
    let mut buf = vec![];
    entry.value.encode(&mut buf, &mut ctx.encoder);
    ctx.query_cache.set(node_index, &buf);
  }

  fn deserialize(&self, _ctx: &mut DeserializeContext<'_>, _entry_id: usize) {
    // TODO: implement deserialization
  }
}

impl<T: StableHash + Send + Sync + 'static> InputFieldIngredient<T> {
  pub fn value_fingerprint(&self, db: &dyn QueryDatabase, arg_id: usize) -> Option<Fingerprint> {
    self.data.get(&arg_id).map(|entry| {
      let mut hasher: StableHasher = StableHasher::new();
      entry.value.stable_hash(db, &mut hasher);
      Fingerprint::from_hasher(hasher)
    })
  }
}
