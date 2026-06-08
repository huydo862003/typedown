//! A salsa-like database for incremental Typedown compilation
//! Salsa is not used so I can fully control the incrementalism + task-based parallelism

/// We need this so macros can be used here and consumer crates
extern crate self as typedown_db;

pub mod derived;
pub mod engine;
#[cfg(test)]
pub(crate) mod fixtures;
pub mod inputs;

pub use engine::*;
/// TIL: Macros that use 3rd-party crates would require that crate to be installed in the consumer crate
/// To workaround this, we would re-export the 3rd-party crate and use a path that references the current crate
pub use inventory;
use typedown_macros::query_db;

#[query_db]
pub struct TypedownDatabase {
  storage: QueryStorage,
}
