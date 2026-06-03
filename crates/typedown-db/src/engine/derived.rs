//! Derived query engine for the incremental database

/// A fast id for a derived state
/// Derived id is bound to a database's lifetime
pub trait DerivedId<'db> {
  /// Marker used by macros to verify a type implements DerivedId at compile time.
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  const __TYPEDOWN_DERIVED_ID: () = ();
}
