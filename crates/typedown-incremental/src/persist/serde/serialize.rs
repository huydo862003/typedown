use std::collections::HashMap;

use crate::{DepId, Encoder, QueryDatabase};
use crate::persist::serialized::dep_graph::{DepNode, DepNodeIndex};
use crate::persist::serialized::query_cache::FooterCacheEntry;

/// Context for serializing ingredients during dump.
/// Accumulates dep graph nodes and streams query result blobs.
pub struct SerializeContext<'a> {
  pub dep_graph: DepGraphBuilder,
  pub query_cache: QueryCacheBuilder,
  pub encoder: Encoder<'a>,
}

impl<'a> SerializeContext<'a> {
  pub fn new<DB: QueryDatabase>(db: &'a DB) -> Self {
    Self {
      dep_graph: DepGraphBuilder::new(),
      query_cache: QueryCacheBuilder::new(),
      encoder: Encoder::new(db),
    }
  }

  pub fn finalize(mut self) -> (Vec<DepNode>, Vec<FooterCacheEntry>) {
    self.dep_graph.resolve_edges();
    (self.dep_graph.nodes, self.query_cache.entries)
  }
}

/// Builder for the dep graph during serialization.
pub struct DepGraphBuilder {
  nodes: Vec<DepNode>,
  index_map: HashMap<DepId, DepNodeIndex>,
  /// Edges stored as raw DepIds, resolved to DepNodeIndices in finalize()
  /// since a derived query may depend on another that has not been serialized yet.
  deferred_edges: Vec<(DepNodeIndex, Vec<DepId>)>,
}

impl DepGraphBuilder {
  fn new() -> Self {
    Self {
      nodes: Vec::new(),
      index_map: HashMap::new(),
      deferred_edges: Vec::new(),
    }
  }

  pub fn set(&mut self, dep_id: DepId, node: DepNode, deps: Vec<DepId>) -> DepNodeIndex {
    let index = self.nodes.len() as DepNodeIndex;
    self.nodes.push(node);
    self.index_map.insert(dep_id, index);
    if !deps.is_empty() {
      self.deferred_edges.push((index, deps));
    }
    index
  }

  pub fn get_node_index(&self, dep_id: &DepId) -> Option<DepNodeIndex> {
    self.index_map.get(dep_id).copied()
  }

  fn resolve_edges(&mut self) {
    for (node_index, deps) in std::mem::take(&mut self.deferred_edges) {
      let edges: Vec<DepNodeIndex> = deps
        .iter()
        .map(|dep_id| {
          *self
            .index_map
            .get(dep_id)
            .unwrap_or_else(|| panic!("unresolved dep edge: ({}, {})", dep_id.0, dep_id.1))
        })
        .collect();
      if let Some(DepNode::DerivedQuery {
        edges: existing, ..
      }) = self.nodes.get_mut(node_index as usize)
      {
        *existing = edges;
      }
    }
  }
}

/// Builder for the query cache during serialization.
/// Records the mapping from dep node index to byte offset in the encoder buffer.
pub struct QueryCacheBuilder {
  entries: Vec<FooterCacheEntry>,
}

impl QueryCacheBuilder {
  fn new() -> Self {
    Self {
      entries: Vec::new(),
    }
  }

  pub fn set(&mut self, node_index: DepNodeIndex, byte_offset: u64) {
    self.entries.push(FooterCacheEntry {
      node_index,
      offset: byte_offset,
    });
  }
}
