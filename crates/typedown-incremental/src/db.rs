use std::any::Any;

use super::storage::QueryStorage;
use crate::persist::serialized::SerializedQueryStorage;
use crate::{Decoder, Encoder};

pub trait QueryDatabase: Any {
  #[doc(hidden)]
  unsafe fn storage(&self) -> &QueryStorage;

  #[doc(hidden)]
  unsafe fn storage_mut(&mut self) -> &mut QueryStorage;
}

/// Extension of QueryDatabase that supports serialization.
pub trait SerializableQueryDatabase: QueryDatabase {
  /// Create an encoder for serializing query data.
  fn encoder(&self) -> Box<dyn Encoder + '_>;

  /// Create a decoder for deserializing query data.
  fn decoder<'a>(&'a self, data: &'a [u8], intern_blobs: &'a [Vec<u8>]) -> Box<dyn Decoder + 'a>;

  /// Serialize the current query storage into the serialized formats.
  fn dump(&self) -> SerializedQueryStorage;

  /// Load query storage from the serialized formats.
  fn load(&self, serialized: &SerializedQueryStorage);
}
