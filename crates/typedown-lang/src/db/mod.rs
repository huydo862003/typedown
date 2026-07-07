pub mod codec;
pub mod derived;
#[cfg(test)]
pub(crate) mod fixtures;
pub mod serde;
pub mod types;
pub mod utils;

pub use codec::*;
pub use typedown_incremental::QueryStorage;
use typedown_incremental::{Decoder, Encoder, SerializableQueryDatabase, query_db};

#[query_db]
#[derive(Clone)]
pub struct TypedownDatabase {
  pub storage: QueryStorage,
}

impl SerializableQueryDatabase for TypedownDatabase {
  fn encoder(&self) -> Box<dyn Encoder + '_> {
    Box::new(TypedownEncoder::new(self))
  }

  fn decoder<'a>(&'a self, data: &'a [u8], intern_blobs: &'a [Vec<u8>]) -> Box<dyn Decoder + 'a> {
    Box::new(TypedownDecoder::new(self, data, intern_blobs))
  }
}
