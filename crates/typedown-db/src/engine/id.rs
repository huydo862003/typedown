//! Base Id trait for all query id types

/// All query ids wrap a usize. This trait exposes it in an object-safe way.
pub trait Id {
  fn as_id(&self) -> usize;
}
