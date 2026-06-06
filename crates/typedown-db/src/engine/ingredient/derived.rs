use std::sync::atomic::{AtomicUsize, Ordering};

use dashmap::DashMap;

use crate::{DerivedId, ExecuteContext, QueryDatabase, QueryStackEntry};

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
#[doc(hidden)]
pub struct DerivedQueryIngredient<DB, K, V: DerivedId> {
  ingredient_index: usize,
  next_arg_id: AtomicUsize,
  query_fn: fn(&DB, K) -> V,
  intern_map: DashMap<K, usize>, // key -> stable arg_id
  #[doc(hidden)]
  pub data: DashMap<usize, QueryState<K, V>>, // arg_id -> state
}

impl<
    DB: QueryDatabase + Send + Sync + 'static,
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
    V: DerivedId + Send + Sync + 'static,
  > DerivedQueryIngredient<DB, K, V>
{
  pub fn new(ingredient_index: usize, query_fn: fn(&DB, K) -> V) -> Self {
    Self {
      ingredient_index,
      next_arg_id: AtomicUsize::new(0),
      query_fn,
      intern_map: DashMap::new(),
      data: DashMap::new(),
    }
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

    // Record dependency for the caller (single place)
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
    storage: &crate::QueryStorage,
    current_revision: usize,
    ingredient_index: usize,
    arg_id: usize,
    arg: K,
  ) -> (V, usize) {
    // Check cache
    if let Some(entry) = self.data.get(&arg_id) {
      match &*entry {
        QueryState::Computed(memo) if memo.verified_at >= current_revision => {
          return (memo.value, memo.changed_at);
        }
        QueryState::Computing => {
          // Check if this entry is in our call stack (cycle detection)
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
          // Stale, run green check
          let changed_at = memo.changed_at;
          drop(entry); // Release the read lock

          if self.green_check(db, arg_id, changed_at) {
            // Green check verified or recomputed + backdated. Entry is now fresh.
            if let Some(entry) = self.data.get(&arg_id)
              && let QueryState::Computed(memo) = &*entry
            {
              return (memo.value, memo.changed_at);
            }
          }
          // green_check returned false, fall through to recompute
        }
      }
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
            cached = Some((memo.value, memo.changed_at));
            return;
          }
          // Save old value and changed_at for backdating after recompute
          old_memo = Some((memo.value, memo.changed_at));
        }
        *state = QueryState::Computing;
      })
      .or_insert(QueryState::Computing);

    if let Some((value, changed_at)) = cached {
      return (value, changed_at);
    }

    // Push to query stack and save parent dependencies
    let parent_dependencies = storage.with_context(|ctx| {
      let ctx = ctx.get_or_insert_with(|| ExecuteContext {
        query_stack: Vec::new(),
        dependencies: Vec::new(),
        disambiguator_map: std::collections::HashMap::new(),
      });
      ctx.query_stack.push(QueryStackEntry {
        ingredient_index,
        arg_id,
      });
      std::mem::take(&mut ctx.dependencies)
    });

    // Reset disambiguator map before re-execution
    storage.with_context(|ctx| {
      if let Some(ctx) = ctx {
        ctx.disambiguator_map.clear();
      }
    });

    // Recompute
    let key = arg.clone();
    let value = (self.query_fn)(db, arg);

    // Collect recorded dependencies, restore parent's, and pop stack
    let dependencies = storage.with_context(|ctx| {
      let ctx = ctx
        .as_mut()
        .expect("context disappeared during query execution");
      let dependencies = std::mem::replace(&mut ctx.dependencies, parent_dependencies);
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
        value,
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
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
    V: DerivedId + Send + Sync + 'static,
  > Ingredient for DerivedQueryIngredient<DB, K, V>
{
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
            let ingredient = &storage.ingredients[dep.ingredient_index];
            if !ingredient.green_check(db, dep.arg_id, dep.changed_at) {
              // Dep reports changed, force it to re-execute
              ingredient.re_execute(db, dep.arg_id);
            }
          }

          // Re-check whether are all deps green
          let all_green = deps.iter().all(|dep| {
            storage.ingredients[dep.ingredient_index].green_check(db, dep.arg_id, dep.changed_at)
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
    let db: &DB = (db as &dyn std::any::Any)
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
}

/// A stamped field value for a derived struct
pub struct StampedDerivedField<T> {
  pub value: T,
  pub changed_at: usize, // The last revision number this one changed
}

/// Ingredient for a derived struct field: maps entry id to stamped value
#[doc(hidden)]
pub struct DerivedFieldIngredient<T> {
  #[doc(hidden)]
  pub data: DashMap<usize, StampedDerivedField<T>>,
}

impl<T> DerivedFieldIngredient<T> {
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  pub const __TYPEDOWN_DERIVED_FIELD_INGREDIENT: () = ();

  pub fn new() -> Self {
    Self {
      data: DashMap::new(),
    }
  }
}

impl<T: Send + Sync + 'static> Ingredient for DerivedFieldIngredient<T> {
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
}
