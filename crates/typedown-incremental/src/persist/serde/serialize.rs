use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use tempfile::tempfile;

use crate::{DepId, Encoder, Fingerprint, QueryDatabase};
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

  pub fn db(&self) -> &dyn QueryDatabase {
    self.encoder.db()
  }

  pub fn finalize(self) -> (Vec<DepNode>, memmap2::Mmap, Vec<Vec<u8>>) {
    let nodes = self.dep_graph.finalize();
    let mmap = self.query_cache.finalize();
    let intern_blobs = self.encoder.finish();
    (nodes, mmap, intern_blobs)
  }
}

/// A dep node with edges stored as unresolved DepIds.
#[derive(Debug, Clone)]
pub enum UnresolvedDepNode {
  DerivedQuery {
    name: Fingerprint,
    key: Fingerprint,
    value: Fingerprint,
    edges: Vec<DepId>,
  },
  DerivedField {
    name: Fingerprint,
    field_index: u8,
    value: Fingerprint,
  },
  InputField {
    name: Fingerprint,
    field_index: u8,
    value: Fingerprint,
  },
  Interned {
    name: Fingerprint,
    blob_index: u32,
  },
}

/// Builder for the dep graph during serialization.
pub struct DepGraphBuilder {
  nodes: Vec<UnresolvedDepNode>,
  index_map: HashMap<DepId, DepNodeIndex>,
}

impl DepGraphBuilder {
  fn new() -> Self {
    Self {
      nodes: Vec::new(),
      index_map: HashMap::new(),
    }
  }

  pub fn set(&mut self, dep_id: DepId, node: UnresolvedDepNode) -> DepNodeIndex {
    let index = self.nodes.len() as DepNodeIndex;
    self.nodes.push(node);
    self.index_map.insert(dep_id, index);
    index
  }

  pub fn get_node_index(&self, dep_id: &DepId) -> Option<DepNodeIndex> {
    self.index_map.get(dep_id).copied()
  }

  /// Resolve edges and return the final dep graph nodes.
  pub fn finalize(self) -> Vec<DepNode> {
    self
      .nodes
      .into_iter()
      .map(|node| match node {
        UnresolvedDepNode::DerivedQuery { name, key, value, edges } => {
          let resolved_edges = edges
            .iter()
            .map(|dep_id| {
              *self
                .index_map
                .get(dep_id)
                .unwrap_or_else(|| panic!("unresolved dep edge: ({}, {})", dep_id.0, dep_id.1))
            })
            .collect();
          DepNode::DerivedQuery { name, key, value, edges: resolved_edges }
        }
        UnresolvedDepNode::DerivedField { name, field_index, value } => {
          DepNode::DerivedField { name, field_index, value }
        }
        UnresolvedDepNode::InputField { name, field_index, value } => {
          DepNode::InputField { name, field_index, value }
        }
        UnresolvedDepNode::Interned { name, blob_index } => {
          DepNode::Interned { name, blob_index }
        }
      })
      .collect()
  }
}

/// Builder for the query cache during serialization.
/// Writes blobs to a tempfile and tracks the mapping from dep node index to byte offset.
pub struct QueryCacheBuilder {
  file: File,
  offset: u64,
  entries: Vec<FooterCacheEntry>,
}

impl QueryCacheBuilder {
  fn new() -> Self {
    use crate::persist::serialized::query_cache::FileHeader;

    let mut file = tempfile().expect("Failed to create tempfile for query cache");
    let header = FileHeader::new();
    file.write_all(&header.to_bytes()).expect("Failed to write query cache header");

    Self {
      file,
      offset: 8, // After the 8-byte header
      entries: Vec::new(),
    }
  }

  /// Write a blob to the backing file. Returns the byte offset where the blob was written.
  pub fn set(&mut self, node_index: DepNodeIndex, blob: &[u8]) -> u64 {
    let byte_offset = self.offset;
    self.file.write_all(blob).expect("Failed to write blob to query cache tempfile");
    self.offset += blob.len() as u64;
    self.entries.push(FooterCacheEntry {
      node_index,
      offset: byte_offset,
    });
    byte_offset
  }

  /// Write the footer and convert the backing tempfile into a read-only mmap.
  pub fn finalize(mut self) -> memmap2::Mmap {
    let footer_pos = self.offset;

    // Write entry count
    self.file.write_all(&(self.entries.len() as u64).to_le_bytes()).expect("Failed to write footer entry count");

    // Write entries
    for entry in &self.entries {
      self.file.write_all(&entry.to_bytes()).expect("Failed to write footer entry");
    }

    // Write footer position as the last 8 bytes
    self.file.write_all(&footer_pos.to_le_bytes()).expect("Failed to write footer position");

    unsafe { memmap2::Mmap::map(&self.file).expect("Failed to mmap query cache tempfile") }
  }
}
