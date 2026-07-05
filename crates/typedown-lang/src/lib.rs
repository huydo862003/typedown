//! Typedown language: syntax, typechecking, and evaluation

// Syntax modules
pub mod ast;
pub mod green;
pub mod lex;
pub mod parse;
pub mod red;

// DB modules
pub mod codec;
pub mod derived;
#[cfg(test)]
pub(crate) mod fixtures;
pub mod serde;
pub mod types;
pub mod utils;

pub use codec::*;
pub use typedown_incremental::QueryStorage;
use typedown_incremental::query_db;

#[query_db]
#[derive(Clone)]
pub struct TypedownDatabase {
  pub storage: QueryStorage,
}
