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
