use std::any::Any;

use super::storage::QueryStorage;
use crate::persist::serialized::SerializedQueryStorage;

pub trait QueryDatabase: Any {
  #[doc(hidden)]
  unsafe fn storage(&self) -> &QueryStorage;

  #[doc(hidden)]
  unsafe fn storage_mut(&mut self) -> &mut QueryStorage;
}

/// Extension of QueryDatabase that supports serialization.
pub trait SerializableQueryDatabase: QueryDatabase {
  /// Serialize the current query storage into the serialized formats.
  fn dump(&self) -> SerializedQueryStorage;

  /// Load query storage from the serialized formats.
  fn load(&self, serialized: &SerializedQueryStorage);
}
