//! Input declaration engine for the incremental database

/// A fast id for an input state
/// Input id is bound to a database's lifetime
pub trait InputId<'db> {
  /// Marker used by macros to verify a type implements InputId at compile time.
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  const __TYPEDOWN_INPUT_ID: () = ();
}
