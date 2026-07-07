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

pub type DepNodeIndex = u32;

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
    bytes[0..4].copy_from_slice(&self.magic);
    bytes[4..8].copy_from_slice(&self.version.to_le_bytes());
    bytes
  }

  pub fn from_bytes(bytes: &[u8; 8]) -> Self {
    let mut magic = [0u8; 4];
    magic.copy_from_slice(&bytes[0..4]);
    let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    FileHeader { magic, version }
  }
}

/// A node in the dep graph
#[derive(Debug, Clone)]
pub enum DepNode {
  /// A derived query invocation (e.g. `vault_config(project)`)
  DerivedQuery {
    name: Fingerprint,
    key: Fingerprint,
    value: Fingerprint,
    edges: Vec<u32>,
  },
  /// A derived struct field (e.g. `VaultConfigResult::version`)
  DerivedField {
    name: Fingerprint,
    field_index: u8,
    value: Fingerprint,
  },
  /// An input field (e.g. `File::handle`). Leaf node.
  InputField {
    name: Fingerprint,
    field_index: u8,
    value: Fingerprint,
  },
}

// Tag bytes for serialization
// Distinguishing among 3 kind of dep nodes
const TAG_DERIVED_QUERY: u8 = 0;
const TAG_DERIVED_FIELD: u8 = 1;
const TAG_INPUT_FIELD: u8 = 2;

// Byte sizes
const TAG_SIZE: usize = std::mem::size_of::<u8>();
const FINGERPRINT_SIZE: usize = std::mem::size_of::<Fingerprint>();
const FIELD_INDEX_SIZE: usize = std::mem::size_of::<u8>();
const EDGE_COUNT_SIZE: usize = std::mem::size_of::<u32>();
const EDGE_SIZE: usize = std::mem::size_of::<u32>();

impl DepNode {
  pub fn name(&self) -> Fingerprint {
    match self {
      DepNode::DerivedQuery { name, .. }
      | DepNode::DerivedField { name, .. }
      | DepNode::InputField { name, .. } => *name,
    }
  }

  pub fn value_fingerprint(&self) -> Fingerprint {
    match self {
      DepNode::DerivedQuery { value, .. }
      | DepNode::DerivedField { value, .. }
      | DepNode::InputField { value, .. } => *value,
    }
  }

  pub fn edges(&self) -> &[u32] {
    match self {
      DepNode::DerivedQuery { edges, .. } => edges,
      DepNode::DerivedField { .. } | DepNode::InputField { .. } => &[],
    }
  }

  /// Serialize
  pub fn to_bytes(&self) -> Vec<u8> {
    let capacity = match self {
      DepNode::DerivedQuery { edges, .. } => {
        TAG_SIZE + // discriminant
        FINGERPRINT_SIZE * 3 + // name + key + value fingerprints
        EDGE_COUNT_SIZE + // length of the edges vector
        edges.len() * EDGE_SIZE // edge entries
      }
      DepNode::DerivedField { .. } => {
        TAG_SIZE + // discriminant
        FINGERPRINT_SIZE * 2 + // name + value fingerprints
        FIELD_INDEX_SIZE // the index of this field within the whole derived struct
      }
      DepNode::InputField { .. } => {
        TAG_SIZE + // discriminant
        FINGERPRINT_SIZE * 2 + // name + value fingerprints
        FIELD_INDEX_SIZE // the index of this field within the whole derived struct
      }
    };
    let mut bytes = Vec::with_capacity(capacity); // preallocated to avoid reallocation overhead
    match self {
      DepNode::DerivedQuery {
        name,
        key,
        value,
        edges,
      } => {
        bytes.push(TAG_DERIVED_QUERY);
        bytes.extend_from_slice(&name.0);
        bytes.extend_from_slice(&key.0);
        bytes.extend_from_slice(&value.0);
        bytes.extend_from_slice(&(edges.len() as u32).to_le_bytes());
        for edge in edges {
          bytes.extend_from_slice(&edge.to_le_bytes());
        }
      }
      DepNode::DerivedField {
        name,
        field_index,
        value,
      } => {
        bytes.push(TAG_DERIVED_FIELD);
        bytes.extend_from_slice(&name.0);
        bytes.push(*field_index);
        bytes.extend_from_slice(&value.0);
      }
      DepNode::InputField {
        name,
        field_index,
        value,
      } => {
        bytes.push(TAG_INPUT_FIELD);
        bytes.extend_from_slice(&name.0);
        bytes.push(*field_index);
        bytes.extend_from_slice(&value.0);
      }
    }
    bytes
  }

  /// Deserialize
  pub fn from_bytes(bytes: &[u8]) -> (Self, usize) {
    let tag = bytes[0]; // the variant of the dep node
    let mut pos = TAG_SIZE; // current decoded offset
    match tag {
      // Derived query node
      TAG_DERIVED_QUERY => {
        // fingerprint of the derived query's name
        let name = Fingerprint(bytes[pos..pos + FINGERPRINT_SIZE].try_into().unwrap());
        pos += FINGERPRINT_SIZE;

        // fingerprint of the derived query's args
        let key = Fingerprint(bytes[pos..pos + FINGERPRINT_SIZE].try_into().unwrap());
        pos += FINGERPRINT_SIZE;

        // fingerprint of the derived query's value
        let value = Fingerprint(bytes[pos..pos + FINGERPRINT_SIZE].try_into().unwrap());
        pos += FINGERPRINT_SIZE;

        // the number of edges connecting this dep node
        let edge_count =
          u32::from_le_bytes(bytes[pos..pos + EDGE_COUNT_SIZE].try_into().unwrap()) as usize;
        pos += EDGE_COUNT_SIZE;

        // decode all the edges
        let mut edges = Vec::with_capacity(edge_count);
        for _ in 0..edge_count {
          edges.push(u32::from_le_bytes(
            bytes[pos..pos + EDGE_SIZE].try_into().unwrap(),
          ));
          pos += EDGE_SIZE;
        }

        (
          DepNode::DerivedQuery {
            name,
            key,
            value,
            edges,
          },
          pos,
        )
      }
      // Derived query field
      TAG_DERIVED_FIELD => {
        // fingerprint of the derived query's name
        let name = Fingerprint(bytes[pos..pos + FINGERPRINT_SIZE].try_into().unwrap());
        pos += FINGERPRINT_SIZE;

        // the index within the derived query struct
        let field_index = bytes[pos];
        pos += FIELD_INDEX_SIZE;

        // the fingerprint of the derived field's value
        let value = Fingerprint(bytes[pos..pos + FINGERPRINT_SIZE].try_into().unwrap());
        pos += FINGERPRINT_SIZE;

        (
          DepNode::DerivedField {
            name,
            field_index,
            value,
          },
          pos,
        )
      }
      // Input query field
      TAG_INPUT_FIELD => {
        // fingerprint of the derived query's name
        let name = Fingerprint(bytes[pos..pos + FINGERPRINT_SIZE].try_into().unwrap());
        pos += FINGERPRINT_SIZE;

        // the index within the derived query struct
        let field_index = bytes[pos];
        pos += FIELD_INDEX_SIZE;

        // the fingerprint of the derived field's value
        let value = Fingerprint(bytes[pos..pos + FINGERPRINT_SIZE].try_into().unwrap());
        pos += FINGERPRINT_SIZE;

        (
          DepNode::InputField {
            name,
            field_index,
            value,
          },
          pos,
        )
      }
      _ => panic!("unknown DepNode tag {tag}"),
    }
  }
}

/// File footer (16 bytes).
#[derive(Debug, Clone, Copy)]
pub struct FileFooter {
  pub total_node_count: u64, // 8 bytes
  pub total_edge_count: u64, // 8 bytes
}

impl FileFooter {
  /// Serialize
  pub fn to_bytes(&self) -> [u8; 16] {
    let mut bytes = [0u8; 16];

    // Encode the node count as little-endian & serialize
    bytes[0..8].copy_from_slice(&self.total_node_count.to_le_bytes());

    // Encode the edge count as little-endian & serialize
    bytes[8..16].copy_from_slice(&self.total_edge_count.to_le_bytes());

    bytes
  }

  /// Deserialize
  pub fn from_bytes(bytes: &[u8; 16]) -> Self {
    FileFooter {
      // Decode the node count from little-endian & deserialize
      total_node_count: u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
      // Decode the edge count from little-endian & deserialize
      total_edge_count: u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
    }
  }
}

/// The dep graph readily for serialization or deserialization into
pub struct DepGraph {
  pub header: FileHeader,
  pub nodes: Vec<DepNode>,
  pub footer: FileFooter,
}

impl DepGraph {
  pub fn node_count(&self) -> usize {
    self.nodes.len()
  }
}
