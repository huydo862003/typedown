use std::collections::HashMap;

use crate::persist::serialized::dep_graph::{DepGraph, DepNode, DepNodeIndex};
use crate::persist::serialized::interned_blobs::InternedBlobs;
use crate::persist::serialized::query_cache::QueryCache;
use crate::{Decoder, Fingerprint, QueryDatabase};

/// Context for deserializing ingredients during load.
pub struct DeserializeContext<'a> {
  pub dep_graph: DepGraphReader,
  pub query_cache: QueryCacheReader,
  pub decoder: Decoder<'a>,
}

impl<'a> DeserializeContext<'a> {
  pub fn new(
    db: &'a dyn QueryDatabase,
    dep_graph: DepGraph,
    query_cache: QueryCache,
    interned_blobs: InternedBlobs,
  ) -> Self {
    let intern_blob_bytes = interned_blobs
      .records
      .iter()
      .map(|r| r.to_bytes())
      .collect();

    Self {
      dep_graph: DepGraphReader::new(dep_graph),
      query_cache: QueryCacheReader::new(query_cache),
      decoder: Decoder::new(db, intern_blob_bytes),
    }
  }

  pub fn db(&self) -> &dyn QueryDatabase {
    self.decoder.db()
  }
}

/// Reader for the dep graph during deserialization.
pub struct DepGraphReader {
  dep_graph: DepGraph,
  /// Ingredient name fingerprint -> list of node indices
  by_name: HashMap<Fingerprint, Vec<DepNodeIndex>>,
}

impl DepGraphReader {
  fn new(dep_graph: DepGraph) -> Self {
    let mut by_name: HashMap<Fingerprint, Vec<DepNodeIndex>> = HashMap::new();
    for (index, node) in dep_graph.nodes.iter().enumerate() {
      by_name
        .entry(node.name())
        .or_default()
        .push(index as DepNodeIndex);
    }
    Self { dep_graph, by_name }
  }

  /// Get all node indices for a given ingredient name
  pub fn get_by_ingredients(&self, name: &Fingerprint) -> &[DepNodeIndex] {
    self.by_name.get(name).map(|v| v.as_slice()).unwrap_or(&[])
  }

  /// Get a dep node by index
  pub fn get(&self, index: DepNodeIndex) -> &DepNode {
    &self.dep_graph.nodes[index as usize]
  }
}

/// Reader for the query cache during deserialization
/// Provides blob lookup by dep node index
pub struct QueryCacheReader {
  query_cache: QueryCache,
}

impl QueryCacheReader {
  fn new(query_cache: QueryCache) -> Self {
    Self { query_cache }
  }

  /// Get the encoded blob for a dep node index
  pub fn get(&self, node_index: DepNodeIndex) -> Option<&[u8]> {
    self.query_cache.get(node_index)
  }
}
