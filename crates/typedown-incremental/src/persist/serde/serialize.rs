use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use tempfile::tempfile;

use crate::persist::serialized::dep_graph::{DepNode, DepNodeIndex};
use crate::persist::serialized::query_cache::FooterCacheEntry;
use crate::{DepId, Encoder, Fingerprint, QueryDatabase};

/// Context for serializing ingredients during dump.
/// Accumulates dep graph nodes and streams query result blobs.
pub struct SerializeContext<'a> {
  pub dep_graph: DepGraphBuilder,
  pub query_cache: QueryCacheBuilder,
  pub encoder: Encoder<'a>,
}

impl<'a> SerializeContext<'a> {
  pub fn new(db: &'a dyn QueryDatabase) -> Self {
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
    let nodes = self.dep_graph.finalize(self.encoder.dep_id_table());
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
    changed_at: u64,
    verified_at: u64,
    edges: Vec<DepId>,
  },
  DerivedField {
    name: Fingerprint,
    field_index: u8,
    value: Fingerprint,
    changed_at: u64,
  },
  InputField {
    name: Fingerprint,
    field_index: u8,
    value: Fingerprint,
    changed_at: u64,
  },
  Interned {
    name: Fingerprint,
    blob_index: u32,
  },
}

/// Builder for the dep graph during serialization.
/// Uses the Encoder's dep_id_table for DepId -> DepNodeIndex mapping.
pub struct DepGraphBuilder {
  nodes: Vec<(DepNodeIndex, UnresolvedDepNode)>,
}

impl DepGraphBuilder {
  fn new() -> Self {
    Self { nodes: Vec::new() }
  }

  /// Add a dep node at the given index (obtained from Encoder::add_dep_id).
  pub fn set(&mut self, index: DepNodeIndex, node: UnresolvedDepNode) {
    self.nodes.push((index, node));
  }

  /// Resolve edges and return the final dep graph nodes, using the Encoder's dep_id_table.
  pub fn finalize(self, dep_id_table: &HashMap<DepId, DepNodeIndex>) -> Vec<DepNode> {
    // Sort by index to ensure correct ordering
    let mut sorted = self.nodes;
    sorted.sort_by_key(|(idx, _)| *idx);

    sorted
      .into_iter()
      .map(|(_, node)| match node {
        UnresolvedDepNode::DerivedQuery {
          name,
          key,
          value,
          changed_at,
          verified_at,
          edges,
        } => {
          let resolved_edges = edges
            .iter()
            .map(|dep_id| *dep_id_table.get(dep_id).expect("unresolved dep edge"))
            .collect();
          DepNode::DerivedQuery {
            name,
            key,
            value,
            changed_at,
            verified_at,
            edges: resolved_edges,
          }
        }
        UnresolvedDepNode::DerivedField {
          name,
          field_index,
          value,
          changed_at,
        } => DepNode::DerivedField {
          name,
          field_index,
          value,
          changed_at,
        },
        UnresolvedDepNode::InputField {
          name,
          field_index,
          value,
          changed_at,
        } => DepNode::InputField {
          name,
          field_index,
          value,
          changed_at,
        },
        UnresolvedDepNode::Interned { name, blob_index } => DepNode::Interned { name, blob_index },
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
    file
      .write_all(&header.to_bytes())
      .expect("Failed to write query cache header");

    Self {
      file,
      offset: 8, // After the 8-byte header
      entries: Vec::new(),
    }
  }

  /// Write a blob to the backing file. Returns the byte offset where the blob was written.
  pub fn set(&mut self, node_index: DepNodeIndex, blob: &[u8]) -> u64 {
    let byte_offset = self.offset;
    self
      .file
      .write_all(blob)
      .expect("Failed to write blob to query cache tempfile");
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
    self
      .file
      .write_all(&(self.entries.len() as u64).to_le_bytes())
      .expect("Failed to write footer entry count");

    // Write entries
    for entry in &self.entries {
      self
        .file
        .write_all(&entry.to_bytes())
        .expect("Failed to write footer entry");
    }

    // Write footer position as the last 8 bytes
    self
      .file
      .write_all(&footer_pos.to_le_bytes())
      .expect("Failed to write footer position");

    unsafe { memmap2::Mmap::map(&self.file).expect("Failed to mmap query cache tempfile") }
  }
}
