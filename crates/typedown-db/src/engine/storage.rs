use std::any::Any;

// TIL: We use DashMap to support high-performance concrruent reads, which fits the workload of IDEs
use dashmap::DashMap;

pub struct QueryStorage {
}

impl QueryStorage {
  /// Marker used by the `query_db` macro to verify the storage field type at compile time.
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  pub const __TYPEDOWN_QUERY_STORAGE: () = ();
}
