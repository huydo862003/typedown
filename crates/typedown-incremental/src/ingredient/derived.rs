use std::{
  any::Any,
  collections::HashMap,
  hash::Hash,
  panic::panic_any,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
};

use crate::persist::serialized::dep_graph::{DepNode, DepNodeIndex};
use crate::{Cancelled, ExecuteContext, QueryStackEntry, QueryStorage};
use crate::{
  Decodable, DepId, DeserializeContext, Encodable, Fingerprint, StableHash, StableHasher,
};
use crate::{DerivedId, QueryDatabase, SerializeContext, UnresolvedDepNode};
use dashmap::DashMap;

use super::Ingredient;

/// A dependency recorded during a derived query execution
#[derive(Clone)]
pub struct Dependency {
  pub ingredient_index: usize, // Which ingredient
  pub arg_id: usize,           // Which entry in that ingredient
  pub changed_at: usize,       // The revision it had when we read it
}

/// A memoized derived query result
pub struct StampedDerivedQuery<K, V: DerivedId> {
  pub key: K,                        // The original key, for re-execution
  pub value: V,                      // The derived struct ID
  pub changed_at: usize,             // Revision when the value last actually changed
  pub verified_at: usize,            // Revision when last confirmed valid
  pub dependencies: Vec<Dependency>, // What this query read during execution
}

/// The state of a query entry in the cache
pub enum QueryState<K, V: DerivedId> {
  /// The query is currently being computed
  Computing,
  /// The query has a cached result
  Computed(StampedDerivedQuery<K, V>),
}

/// Ingredient for a derived query function: maps key tuple to memoized result
#[derive(Clone)]
#[doc(hidden)]
pub struct DerivedQueryIngredient<DB, K, V: DerivedId> {
  ingredient_index: usize,
  stable_name: Fingerprint,
  next_arg_id: Arc<AtomicUsize>,
  query_fn: fn(&DB, K) -> V,
  intern_map: Arc<DashMap<K, usize>>, // key -> stable arg_id
  #[doc(hidden)]
  pub data: Arc<DashMap<usize, QueryState<K, V>>>, // arg_id -> state
}

impl<
  DB: QueryDatabase + Send + Sync + 'static,
  K: StableHash + Encodable + Decodable + Eq + Hash + Clone + Send + Sync + 'static,
  V: StableHash + Encodable + Decodable + DerivedId + Clone + PartialEq + Send + Sync + 'static,
> DerivedQueryIngredient<DB, K, V>
{
  pub fn new(ingredient_index: usize, stable_name: &str, query_fn: fn(&DB, K) -> V) -> Self {
    Self {
      ingredient_index,
      stable_name: Fingerprint::from_name(stable_name),
      next_arg_id: Arc::new(AtomicUsize::new(0)),
      query_fn,
      intern_map: Arc::new(DashMap::new()),
      data: Arc::new(DashMap::new()),
    }
  }

  pub fn key_fingerprint(&self, db: &dyn QueryDatabase, arg_id: usize) -> Option<Fingerprint>
  where
    K: StableHash,
  {
    let db = (db as &dyn Any)
      .downcast_ref::<DB>()
      .expect("database type mismatch in key_fingerprint");
    if let Some(entry) = self.data.get(&arg_id) {
      if let QueryState::Computed(memo) = &*entry {
        let mut hasher: StableHasher = StableHasher::new();
        memo.key.stable_hash(db, &mut hasher);
        return Some(Fingerprint::from_hasher(hasher));
      }
    }
    None
  }

  pub fn value_fingerprint(&self, db: &dyn QueryDatabase, arg_id: usize) -> Option<Fingerprint>
  where
    V: StableHash,
  {
    let db = (db as &dyn Any)
      .downcast_ref::<DB>()
      .expect("database type mismatch in value_fingerprint");
    if let Some(entry) = self.data.get(&arg_id) {
      if let QueryState::Computed(memo) = &*entry {
        let mut hasher: StableHasher = StableHasher::new();
        memo.value.stable_hash(db, &mut hasher);
        return Some(Fingerprint::from_hasher(hasher));
      }
    }
    None
  }

  /// Try to load a cached result from the serialized cache.
  fn try_load_from_serialized(
    &self,
    db: &DB,
    storage: &QueryStorage,
    arg: &K,
  ) -> Option<(V, usize)> {
    let ctx = storage.deserialize_ctx.as_ref().as_ref()?;

    // Compute key fingerprint to find the matching node
    let mut hasher = StableHasher::new();
    arg.stable_hash(db, &mut hasher);
    let key_fp = Fingerprint::from_hasher(hasher);

    let (node_index, node) = ctx.find_derived_query(self.stable_name, key_fp)?;

    let DepNode::DerivedQuery {
      changed_at, edges, ..
    } = node
    else {
      return None;
    };

    // Green check: compare multisets of (ingredient_name, value_fingerprint).
    // Both the serialized edges and current entries must have matching counts.
    let mut expected: HashMap<(Fingerprint, Fingerprint), usize> = HashMap::new();
    for edge_idx in edges {
      let edge_node = &ctx.serialized.dep_graph.nodes[*edge_idx as usize];
      *expected
        .entry((edge_node.name(), edge_node.value_fingerprint()))
        .or_default() += 1;
    }

    let mut actual: HashMap<(Fingerprint, Fingerprint), usize> = HashMap::new();
    for (name, _) in expected.keys() {
      for entry in storage.ingredients.iter() {
        if entry.ingredient.name() != *name {
          continue;
        }
        for eid in entry.ingredient.entry_ids() {
          if let Some(fp) = entry.ingredient.value_fingerprint(db, eid) {
            *actual.entry((*name, fp)).or_default() += 1;
          }
        }
      }
    }

    for (key, &needed) in &expected {
      let available = actual.get(key).copied().unwrap_or(0);
      if available < needed {
        return None;
      }
    }

    // All deps green. Decode the cached result.
    let decoder = ctx.decoder(db);
    let blob = ctx.serialized.query_cache.get(node_index)?;
    let mut data: &[u8] = blob;
    let _key = K::decode(&mut data, &decoder);
    let value = V::decode(&mut data, &decoder);

    Some((value, *changed_at as usize))
  }

  /// Get or create a stable entry ID for a key
  fn get_or_intern_arg(&self, arg: &K) -> usize {
    if let Some(entry) = self.intern_map.get(arg) {
      return *entry.value();
    }
    let arg_id = self.next_arg_id.fetch_add(1, Ordering::Relaxed);
    *self.intern_map.entry(arg.clone()).or_insert(arg_id).value()
  }

  /// Execute a derived query: returns cached result if valid, otherwise runs the query function
  pub fn execute_query(&self, db: &DB, arg: K) -> V {
    let storage = unsafe { db.storage() };
    let current_revision = storage.revision.load(Ordering::Acquire);
    let ingredient_index = self.ingredient_index;
    let arg_id = self.get_or_intern_arg(&arg);

    let (value, changed_at) =
      self.execute_query_inner(db, storage, current_revision, ingredient_index, arg_id, arg);

    // Record dependency for the caller
    storage.with_context(|ctx| {
      if let Some(ctx) = ctx {
        ctx.dependencies.push(Dependency {
          ingredient_index,
          arg_id,
          changed_at,
        });
      }
    });

    value
  }

  /// Inner implementation that returns (value, changed_at)
  fn execute_query_inner(
    &self,
    db: &DB,
    storage: &QueryStorage,
    current_revision: usize,
    ingredient_index: usize,
    arg_id: usize,
    arg: K,
  ) -> (V, usize) {
    // Check cache
    if let Some(entry) = self.data.get(&arg_id) {
      match &*entry {
        QueryState::Computed(memo) if memo.verified_at >= current_revision => {
          return (memo.value.clone(), memo.changed_at);
        }
        QueryState::Computing => {
          // Cycle detection: Check if this entry is in our call stack
          let is_cycle = storage.with_context(|ctx| {
            ctx.as_ref().is_some_and(|ctx| {
              ctx
                .query_stack
                .iter()
                .any(|e| e.ingredient_index == ingredient_index && e.arg_id == arg_id)
            })
          });
          if is_cycle {
            panic!("cycle detected in derived query");
          }
          // Not in our stack: another thread is computing this, compute anyway, which should be negligible
          // Don't wait here, else you risk deadlock
        }
        QueryState::Computed(memo) => {
          // Stale compared to current revision (not sure if real stale)
          // Run green check
          let changed_at = memo.changed_at;
          drop(entry); // Release the read lock

          if self.green_check(db, arg_id, changed_at) {
            // The green check has verified or
            // recomputed + backdated so the entry must now be fresh
            if let Some(entry) = self.data.get(&arg_id)
              && let QueryState::Computed(memo) = &*entry
            {
              return (memo.value.clone(), memo.changed_at);
            }
          }
          // green_check returned false, need to recompute
        }
      }
    }

    // Try loading from previous session before recomputing
    if let Some(result) = self.try_load_from_serialized(db, storage, &arg) {
      let (value, changed_at) = result;
      return (value, changed_at);
    }

    #[allow(unused_labels)]
    'Time_A: {}

    #[allow(unused_labels)]
    'Time_B: {}

    // Mark as computing
    // This can override a fresh computed value between 'Time_A and 'Time_B
    // But it should not matter except for a little redundant work:
    // - Everything is immutable, so recomputation is fine
    // - If a thread computes the value then see a stale value again, it would just trigger recompute (redundant work), but it doesn't cause any cycle
    // - The thread that computes the value still return the fresh value

    // EDIT: The current optimization (?) is to use a shard lock provided by DashMap to check if the value is overrided with a fresh value already to skip unnecessary computation
    // However, this introduces a lock, so I don't really know
    let mut cached = None;
    let mut old_memo = None;
    self
      .data
      .entry(arg_id)
      .and_modify(|state| {
        if let QueryState::Computed(memo) = state {
          if memo.verified_at >= current_revision {
            cached = Some((memo.value.clone(), memo.changed_at));
            return;
          }
          // Save old value and changed_at for backdating after recompute
          old_memo = Some((memo.value.clone(), memo.changed_at));
        }
        *state = QueryState::Computing;
      })
      .or_insert(QueryState::Computing);

    if let Some((value, changed_at)) = cached {
      return (value, changed_at);
    }

    // Push to query stack, save parent dependencies and disambiguator state
    let (parent_dependencies, parent_disambiguator_map) = storage.with_context(|ctx| {
      let ctx = ctx.get_or_insert_with(|| ExecuteContext {
        query_stack: Vec::new(),
        dependencies: Vec::new(),
        disambiguator_map: std::collections::HashMap::new(),
      });
      ctx.query_stack.push(QueryStackEntry {
        ingredient_index,
        arg_id,
      });
      (
        std::mem::take(&mut ctx.dependencies),
        std::mem::take(&mut ctx.disambiguator_map),
      )
    });

    // Check for cancellation before recomputing
    let storage = unsafe { db.storage() };
    if storage.cancelled.load(Ordering::Relaxed) {
      panic_any(Cancelled);
    }

    // Recompute
    let key = arg.clone();
    let value = (self.query_fn)(db, arg);

    // Collect recorded dependencies, restore parent state, and pop stack
    let dependencies = storage.with_context(|ctx| {
      let ctx = ctx
        .as_mut()
        .expect("context disappeared during query execution");
      let dependencies = std::mem::replace(&mut ctx.dependencies, parent_dependencies);
      ctx.disambiguator_map = parent_disambiguator_map;
      ctx.query_stack.pop();
      dependencies
    });

    // Backdating: if the new value equals the old, keep the old changed_at
    // This prevents unnecessary invalidation of downstream queries
    let changed_at = match old_memo {
      Some((old_value, old_changed_at)) if old_value == value => old_changed_at,
      _ => current_revision,
    };

    // Store the result
    self.data.insert(
      arg_id,
      QueryState::Computed(StampedDerivedQuery {
        key,
        value: value.clone(),
        changed_at,
        verified_at: current_revision,
        dependencies,
      }),
    );

    (value, changed_at)
  }
}

impl<
  DB: QueryDatabase + Send + Sync + 'static,
  K: StableHash + Encodable + Decodable + Eq + Hash + Clone + Send + Sync + 'static,
  V: StableHash + Encodable + Decodable + DerivedId + Clone + PartialEq + Send + Sync + 'static,
> Ingredient for DerivedQueryIngredient<DB, K, V>
{
  fn name(&self) -> Fingerprint {
    self.stable_name
  }

  /// Check the red-green algo here: https://rustc-dev-guide.rust-lang.org/queries/incremental-compilation-in-detail.html#improving-accuracy-the-red-green-algorithm
  /// We're similar in idea
  fn green_check(&self, db: &dyn QueryDatabase, arg_id: usize, last_changed_at: usize) -> bool {
    let storage = unsafe { db.storage() };
    let current_revision = storage.revision.load(Ordering::Acquire);

    match self.data.get(&arg_id) {
      Some(entry) => match &*entry {
        QueryState::Computed(memo) => {
          if memo.verified_at >= current_revision {
            return memo.changed_at <= last_changed_at;
          }
          // Stale: re-execute deps so they can backdate, then re-check
          let deps = memo.dependencies.clone();
          drop(entry);

          for dep in &deps {
            let ingredient = &storage.ingredients[dep.ingredient_index].ingredient;
            if !ingredient.green_check(db, dep.arg_id, dep.changed_at) {
              // Dep reports changed, force it to re-execute
              ingredient.re_execute(db, dep.arg_id);
            }
          }

          // Re-check whether are all deps green
          let all_green = deps.iter().all(|dep| {
            storage.ingredients[dep.ingredient_index]
              .ingredient
              .green_check(db, dep.arg_id, dep.changed_at)
          });

          if all_green {
            // Bump verified_at
            if let Some(mut entry) = self.data.get_mut(&arg_id)
              && let QueryState::Computed(memo) = &mut *entry
            {
              memo.verified_at = current_revision;
              return memo.changed_at <= last_changed_at;
            }
          }
          false
        }
        QueryState::Computing => false, // conservatively assume changed
      },
      None => false,
    }
  }

  fn re_execute(&self, db: &dyn QueryDatabase, arg_id: usize) {
    let db: &DB = (db as &dyn Any)
      .downcast_ref::<DB>()
      .expect("database type mismatch in re_execute");
    // Look up the key from the memo and re-execute
    if let Some(entry) = self.data.get(&arg_id) {
      if let QueryState::Computed(memo) = &*entry {
        let key = memo.key.clone();
        drop(entry);
        self.execute_query(db, key);
      }
    }
  }

  fn entry_ids(&self) -> Box<dyn Iterator<Item = usize> + '_> {
    Box::new(self.data.iter().map(|entry| *entry.key()))
  }

  fn value_fingerprint(&self, db: &dyn QueryDatabase, entry_id: usize) -> Option<Fingerprint> {
    DerivedQueryIngredient::value_fingerprint(self, db, entry_id)
  }

  fn deserialize(&self, _ctx: &DeserializeContext, _node_index: DepNodeIndex) -> Option<DepId> {
    todo!()
  }

  fn serialize(&self, ctx: &mut SerializeContext, entry_id: usize) {
    let Some(entry) = self.data.get(&entry_id) else {
      return;
    };
    let QueryState::Computed(memo) = &*entry else {
      return;
    };

    // Collect dependency edges as DepIds
    let edges = memo
      .dependencies
      .iter()
      .map(|dep| (dep.ingredient_index, dep.arg_id))
      .collect();

    let dep_id = (self.ingredient_index, entry_id);
    let node_index = ctx.encoder.add_dep_id(dep_id);
    ctx.dep_graph.set(
      node_index,
      UnresolvedDepNode::DerivedQuery {
        name: self.stable_name,
        key: self
          .key_fingerprint(ctx.db(), entry_id)
          .expect("Computed entry must have a key fingerprint"),
        value: self
          .value_fingerprint(ctx.db(), entry_id)
          .expect("Computed entry must have a value fingerprint"),
        changed_at: memo.changed_at as u64,
        verified_at: memo.verified_at as u64,
        edges,
      },
    );

    // Encode key and value into the query cache
    let mut buf = vec![];
    memo.key.encode(&mut buf, &mut ctx.encoder);
    memo.value.encode(&mut buf, &mut ctx.encoder);
    ctx.query_cache.set(node_index, &buf);
  }
}

/// A stamped field value for a derived struct
pub struct StampedDerivedField<T> {
  pub value: T,
  pub changed_at: usize, // The last revision number this one changed
}

/// Ingredient for a derived struct field: maps entry id to stamped value
#[derive(Clone)]
#[doc(hidden)]
pub struct DerivedFieldIngredient<T> {
  ingredient_index: usize,
  field_index: u8,
  name: &'static str,
  #[doc(hidden)]
  pub data: Arc<DashMap<usize, StampedDerivedField<T>>>,
}

impl<T> DerivedFieldIngredient<T> {
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  pub const __TYPEDOWN_DERIVED_FIELD_INGREDIENT: () = ();

  pub fn new(ingredient_index: usize, name: &'static str, field_index: u8) -> Self {
    Self {
      ingredient_index,
      field_index,
      name,
      data: Arc::new(DashMap::new()),
    }
  }
}

impl<T: StableHash + Send + Sync + 'static> Ingredient for DerivedFieldIngredient<T> {
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
    // Derived fields are set by the query, nothing to recompute
  }

  fn entry_ids(&self) -> Box<dyn Iterator<Item = usize> + '_> {
    Box::new(self.data.iter().map(|entry| *entry.key()))
  }

  fn value_fingerprint(&self, db: &dyn QueryDatabase, entry_id: usize) -> Option<Fingerprint> {
    DerivedFieldIngredient::value_fingerprint(self, db, entry_id)
  }

  fn deserialize(&self, _ctx: &DeserializeContext, _node_index: DepNodeIndex) -> Option<DepId> {
    todo!()
  }

  fn serialize(&self, ctx: &mut SerializeContext, entry_id: usize) {
    let Some(entry) = self.data.get(&entry_id) else {
      return;
    };

    // Only register a dep node; the value blob lives in the parent query's cache entry.
    let dep_id = (self.ingredient_index, entry_id);
    let node_index = ctx.encoder.add_dep_id(dep_id);
    ctx.dep_graph.set(
      node_index,
      UnresolvedDepNode::DerivedField {
        name: self.name(),
        field_index: self.field_index,
        value: self
          .value_fingerprint(ctx.db(), entry_id)
          .expect("Entry is available so there must be a fingerprint"),
        changed_at: entry.changed_at as u64,
      },
    );
  }
}

impl<T: StableHash + Send + Sync + 'static> DerivedFieldIngredient<T> {
  pub fn value_fingerprint(&self, db: &dyn QueryDatabase, arg_id: usize) -> Option<Fingerprint> {
    self.data.get(&arg_id).map(|entry| {
      let mut hasher: StableHasher = StableHasher::new();
      entry.value.stable_hash(db, &mut hasher);
      Fingerprint::from_hasher(hasher)
    })
  }
}
