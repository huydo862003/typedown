use std::collections::HashMap;

use crate::persist::serialized::dep_graph::{DepNode, DepNodeIndex};
use crate::persist::serialized::query_cache::FooterCacheEntry;
use crate::{Decoder, DepId, Encoder, Fingerprint};

/// Context for serializing ingredients during dump.
/// Accumulates dep graph nodes and streams query result blobs.
pub struct SerializeContext<'a> {
  encoder: &'a mut dyn Encoder,
  dep_nodes: Vec<DepNode>,
  /// DepId -> dep graph node index
  node_index_map: HashMap<DepId, DepNodeIndex>,
  /// Maps each dep node to its result blob's byte offset in the encoder buffer
  query_cache_entries: Vec<FooterCacheEntry>,
  /// DerivedQuery edges stored as raw DepIds, resolved to DepNodeIndices
  /// after all ingredients serialize (since a derived query may depend on another
  /// derived query that has not been serialized yet).
  deferred_edges: Vec<(DepNodeIndex, Vec<DepId>)>,
}

impl<'a> SerializeContext<'a> {
  pub fn new(encoder: &'a mut dyn Encoder) -> Self {
    Self {
      encoder,
      dep_nodes: Vec::new(),
    }
  }
}

/// Context for deserializing ingredients during load.
/// Provides access to the previously serialized data.
pub struct DeserializeContext<'a> {
  pub decoder: &'a mut dyn Decoder,
}
