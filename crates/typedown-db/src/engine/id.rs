//! Base Id trait for all query id types

/// All query ids wrap a usize. This trait exposes it in an object-safe way.
/// Returns (type_tag, id) where type_tag is the ingredient start index
/// (unique per type) and id is the instance id within that type.
pub trait Id {
  fn as_id(&self) -> (usize, usize);
}
