//! A salsa-like database for incremental Typedown compilation
//! Salsa is not used so I can fully control the incrementalism + task-based parallelism
extern crate self as typedown_db;
pub mod derived;
pub mod engine;
pub mod inputs;

pub use engine::*;
use typedown_macros::query_db;

#[query_db]
pub struct TypedownDatabase {
  storage: QueryStorage,
}
