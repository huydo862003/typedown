use std::sync::Arc;

use dashmap::DashMap;

use crate::{
  Encodable, Fingerprint, QueryDatabase, SerializeContext, StableHash, StableHasher,
  UnresolvedDepNode,
};

use super::Ingredient;

/// An ingredient for an interned struct
#[derive(Clone)]
#[doc(hidden)]
pub struct InternedIngredient<T> {
  ingredient_index: usize,
  name: &'static str,
  #[doc(hidden)]
  pub data: Arc<DashMap<usize, T>>,
}

impl<T> InternedIngredient<T> {
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  pub const __TYPEDOWN_INTERNED_INGREDIENT: () = ();

  pub fn new(ingredient_index: usize, name: &'static str) -> Self {
    Self {
      ingredient_index,
      name,
      data: Arc::new(DashMap::new()),
    }
  }
}

impl<T: StableHash + Send + Sync + 'static> InternedIngredient<T> {
  pub fn value_fingerprint(&self, db: &dyn QueryDatabase, arg_id: usize) -> Option<Fingerprint> {
    self.data.get(&arg_id).map(|entry| {
      let mut hasher = StableHasher::new();
      entry.value().stable_hash(db, &mut hasher);
      Fingerprint::from_hasher(hasher)
    })
  }
}

impl<T: StableHash + Encodable + Send + Sync + 'static> Ingredient for InternedIngredient<T> {
  fn name(&self) -> Fingerprint {
    Fingerprint::from_name(self.name)
  }

  fn green_check(&self, _db: &dyn QueryDatabase, _arg_id: usize, _last_changed_at: usize) -> bool {
    // Interned values never change, always green
    true
  }

  fn re_execute(&self, _db: &dyn QueryDatabase, _arg_id: usize) {
    // Interned values never change, nothing to recompute
  }

  fn entry_ids(&self) -> Box<dyn Iterator<Item = usize> + '_> {
    Box::new(self.data.iter().map(|entry| *entry.key()))
  }

  fn value_fingerprint(&self, db: &dyn QueryDatabase, entry_id: usize) -> Option<Fingerprint> {
    InternedIngredient::value_fingerprint(self, db, entry_id)
  }

  fn serialize(&self, ctx: &mut SerializeContext, entry_id: usize) {
    let Some(entry) = self.data.get(&entry_id) else {
      return;
    };

    // Encode the value to register it in the encoder's intern table
    let mut buf = vec![];
    entry.value().encode(&mut buf, &mut ctx.encoder);
    let blob_index = ctx.encoder.intern_blob(buf, Some(entry_id));

    let dep_id = (self.ingredient_index, entry_id);
    let node_index = ctx.encoder.add_dep_id(dep_id);
    ctx.dep_graph.set(
      node_index,
      UnresolvedDepNode::Interned {
        name: self.name(),
        blob_index,
      },
    );
  }
}
