//! On-disk format for the dependency graph (`dep-graph.bin`).
//! Based on rustc's dep-graph.bin
//!
//! Stores nodes, edges, and fingerprints. No query result data.
//! Fully decoded upfront into memory on load.
//!
//! ```text
//! [ FileHeader (8 bytes)       ]  magic "TDEP" + version
//! [ DepNode                    ]  tag + variant-specific data
//! [ DepNode                    ]
//! [ ...                        ]
//! [ FileFooter (16 bytes)      ]  total_node_count + total_edge_count
//! ```

use crate::Fingerprint;
use crate::types::DepNodeKind;

/* File identification */
// Magic: 4 bytes
// Version: 4 bytes
// Together takes 8 bytes

pub const MAGIC: [u8; 4] = *b"TDEP"; // Magic bytes, like PNG/ELF magic bytes
pub const VERSION: u32 = 1;

/// File header (8 bytes).
#[derive(Debug, Clone, Copy)]
pub struct FileHeader {
  pub magic: [u8; 4],
  pub version: u32,
}

impl FileHeader {
  pub fn new() -> Self {
    FileHeader {
      magic: MAGIC,
      version: VERSION,
    }
  }

  /// A best-effort check if the read file is valid
  pub fn is_valid(&self) -> bool {
    self.magic == MAGIC && self.version == VERSION
  }

  /// Serialize
  pub fn to_bytes(&self) -> [u8; 8] {
    let mut bytes = [0u8; 8];
    bytes[0..4].copy_from_slice(&self.magic); // Char need not further endian processing
    bytes[4..8].copy_from_slice(&self.version.to_le_bytes()); // Encode as little endian
    bytes
  }

  pub fn from_bytes(bytes: &[u8; 8]) -> Self {
    let mut magic = [0u8; 4];
    magic.copy_from_slice(&bytes[0..4]);
    let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    FileHeader { magic, version }
  }
}

/// A node in the dep graph. Each variant stores only what it needs.
#[derive(Debug, Clone)]
pub enum DepNode {
  /// A derived query invocation (e.g. `vault_config(project)`)
  DerivedQuery {
    kind: DepNodeKind,
    key: Fingerprint,
    value: Fingerprint,
    edges: Vec<u32>,
  },
  /// A derived struct field (e.g. `VaultConfigResult::version`)
  DerivedField {
    field_index: u8,
    value: Fingerprint,
    edge: u32,
  },
  /// An input entity (e.g. `File`)
  InputQuery {
    kind: DepNodeKind,
    value: Fingerprint,
  },
  /// An input field (e.g. `File::handle`)
  InputField {
    field_index: u8,
    value: Fingerprint,
    edge: u32,
  },
}

// Tag bytes for serialization
const TAG_DERIVED_QUERY: u8 = 0;
const TAG_DERIVED_FIELD: u8 = 1;
const TAG_INPUT_QUERY: u8 = 2;
const TAG_INPUT_FIELD: u8 = 3;

// Byte sizes
const FINGERPRINT_SIZE: usize = std::mem::size_of::<Fingerprint>();
const TAG_SIZE: usize = std::mem::size_of::<u8>();
const KIND_SIZE: usize = std::mem::size_of::<u8>();
const FIELD_INDEX_SIZE: usize = std::mem::size_of::<u8>();
const EDGE_COUNT_SIZE: usize = std::mem::size_of::<u32>();
const EDGE_SIZE: usize = std::mem::size_of::<u32>();

impl DepNode {
  pub fn kind(&self) -> Option<DepNodeKind> {
    match self {
      DepNode::DerivedQuery { kind, .. } | DepNode::InputQuery { kind, .. } => Some(*kind),
      DepNode::DerivedField { .. } | DepNode::InputField { .. } => None,
    }
  }

  pub fn value_fingerprint(&self) -> Fingerprint {
    match self {
      DepNode::DerivedQuery { value, .. }
      | DepNode::DerivedField { value, .. }
      | DepNode::InputQuery { value, .. }
      | DepNode::InputField { value, .. } => *value,
    }
  }

  pub fn edges(&self) -> &[u32] {
    match self {
      DepNode::DerivedQuery { edges, .. } => edges,
      DepNode::DerivedField { edge, .. } | DepNode::InputField { edge, .. } => {
        std::slice::from_ref(edge)
      }
      DepNode::InputQuery { .. } => &[],
    }
  }

  pub fn to_bytes(&self) -> Vec<u8> {
    let capacity = match self {
      DepNode::DerivedQuery { edges, .. } => {
        TAG_SIZE + KIND_SIZE + FINGERPRINT_SIZE * 2 + EDGE_COUNT_SIZE + edges.len() * EDGE_SIZE
      }
      DepNode::DerivedField { .. } | DepNode::InputField { .. } => {
        TAG_SIZE + FIELD_INDEX_SIZE + FINGERPRINT_SIZE + EDGE_SIZE
      }
      DepNode::InputQuery { .. } => TAG_SIZE + KIND_SIZE + FINGERPRINT_SIZE,
    };
    let mut bytes = Vec::with_capacity(capacity);
    match self {
      DepNode::DerivedQuery {
        kind,
        key,
        value,
        edges,
      } => {
        bytes.push(TAG_DERIVED_QUERY);
        bytes.push(*kind as u8);
        bytes.extend_from_slice(&key.0);
        bytes.extend_from_slice(&value.0);
        bytes.extend_from_slice(&(edges.len() as u32).to_le_bytes());
        for edge in edges {
          bytes.extend_from_slice(&edge.to_le_bytes());
        }
      }
      DepNode::DerivedField {
        field_index,
        value,
        edge,
      } => {
        bytes.push(TAG_DERIVED_FIELD);
        bytes.push(*field_index);
        bytes.extend_from_slice(&value.0);
        bytes.extend_from_slice(&edge.to_le_bytes());
      }
      DepNode::InputQuery { kind, value } => {
        bytes.push(TAG_INPUT_QUERY);
        bytes.push(*kind as u8);
        bytes.extend_from_slice(&value.0);
      }
      DepNode::InputField {
        field_index,
        value,
        edge,
      } => {
        bytes.push(TAG_INPUT_FIELD);
        bytes.push(*field_index);
        bytes.extend_from_slice(&value.0);
        bytes.extend_from_slice(&edge.to_le_bytes());
      }
    }
    bytes
  }

  pub fn from_bytes(bytes: &[u8]) -> (Self, usize) {
    let tag = bytes[0];
    let mut pos = TAG_SIZE;
    match tag {
      TAG_DERIVED_QUERY => {
        let kind_byte = bytes[pos];
        pos += KIND_SIZE;
        let kind = DepNodeKind::try_from(kind_byte)
          .unwrap_or_else(|_| panic!("unknown DepNodeKind {kind_byte}"));
        let key = Fingerprint(bytes[pos..pos + FINGERPRINT_SIZE].try_into().unwrap());
        pos += FINGERPRINT_SIZE;
        let value = Fingerprint(bytes[pos..pos + FINGERPRINT_SIZE].try_into().unwrap());
        pos += FINGERPRINT_SIZE;
        let edge_count =
          u32::from_le_bytes(bytes[pos..pos + EDGE_COUNT_SIZE].try_into().unwrap()) as usize;
        pos += EDGE_COUNT_SIZE;
        let mut edges = Vec::with_capacity(edge_count);
        for _ in 0..edge_count {
          edges.push(u32::from_le_bytes(
            bytes[pos..pos + EDGE_SIZE].try_into().unwrap(),
          ));
          pos += EDGE_SIZE;
        }
        (
          DepNode::DerivedQuery {
            kind,
            key,
            value,
            edges,
          },
          pos,
        )
      }
      TAG_DERIVED_FIELD => {
        let field_index = bytes[pos];
        pos += FIELD_INDEX_SIZE;
        let value = Fingerprint(bytes[pos..pos + FINGERPRINT_SIZE].try_into().unwrap());
        pos += FINGERPRINT_SIZE;
        let edge = u32::from_le_bytes(bytes[pos..pos + EDGE_SIZE].try_into().unwrap());
        pos += EDGE_SIZE;
        (
          DepNode::DerivedField {
            field_index,
            value,
            edge,
          },
          pos,
        )
      }
      TAG_INPUT_FIELD => {
        let field_index = bytes[pos];
        pos += FIELD_INDEX_SIZE;
        let value = Fingerprint(bytes[pos..pos + FINGERPRINT_SIZE].try_into().unwrap());
        pos += FINGERPRINT_SIZE;
        let edge = u32::from_le_bytes(bytes[pos..pos + EDGE_SIZE].try_into().unwrap());
        pos += EDGE_SIZE;
        (
          DepNode::InputField {
            field_index,
            value,
            edge,
          },
          pos,
        )
      }
      TAG_INPUT_QUERY => {
        let kind_byte = bytes[pos];
        pos += KIND_SIZE;
        let kind = DepNodeKind::try_from(kind_byte)
          .unwrap_or_else(|_| panic!("unknown DepNodeKind {kind_byte}"));
        let value = Fingerprint(bytes[pos..pos + FINGERPRINT_SIZE].try_into().unwrap());
        pos += FINGERPRINT_SIZE;
        (DepNode::InputQuery { kind, value }, pos)
      }
      _ => panic!("unknown DepNode tag {tag}"),
    }
  }
}

/// File footer (16 bytes).
#[derive(Debug, Clone, Copy)]
pub struct FileFooter {
  pub total_node_count: u64,
  pub total_edge_count: u64,
}

impl FileFooter {
  pub fn to_bytes(&self) -> [u8; 16] {
    let mut bytes = [0u8; 16];
    bytes[0..8].copy_from_slice(&self.total_node_count.to_le_bytes());
    bytes[8..16].copy_from_slice(&self.total_edge_count.to_le_bytes());
    bytes
  }

  pub fn from_bytes(bytes: &[u8; 16]) -> Self {
    FileFooter {
      total_node_count: u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
      total_edge_count: u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
    }
  }
}

/// Previous session's dep graph, decoded from `dep-graph.bin`.
pub struct SerializedDepGraph {
  pub header: FileHeader,
  pub nodes: Vec<DepNode>,
  pub footer: FileFooter,
}

impl SerializedDepGraph {
  pub fn node_count(&self) -> usize {
    self.nodes.len()
  }
}
