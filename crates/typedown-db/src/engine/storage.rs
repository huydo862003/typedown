pub struct QueryStorage {}

impl QueryStorage {
  /// Marker used by the `query_db` macro to verify the storage field type at compile time.
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  pub const __TYPEDOWN_QUERY_STORAGE: () = ();
}
