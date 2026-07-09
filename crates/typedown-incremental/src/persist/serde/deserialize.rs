use std::collections::HashMap;
use std::sync::OnceLock;

use crate::persist::serialized::SerializedQueryStorage;
use crate::persist::serialized::dep_graph::{DepNode, DepNodeIndex};
use crate::{Decoder, Fingerprint, QueryDatabase};

/// All state needed for lazy deserialization from a previous session.
pub struct DeserializeContext {
  pub serialized: SerializedQueryStorage,
  intern_blobs: Vec<Vec<u8>>,
  fingerprint_map: OnceLock<HashMap<Fingerprint, Vec<DepNodeIndex>>>,
}

impl DeserializeContext {
  pub fn new(serialized: SerializedQueryStorage) -> Self {
    let intern_blobs = serialized
      .interned_blobs
      .records
      .iter()
      .map(|r| r.to_bytes())
      .collect();
    Self {
      serialized,
      intern_blobs,
      fingerprint_map: OnceLock::new(),
    }
  }

  /// Create a Decoder bound to the given database.
  pub fn decoder<'a>(&'a self, db: &'a dyn QueryDatabase) -> Decoder<'a> {
    Decoder::new(db, &self.intern_blobs)
  }

  /// Lazily-built index: ingredient name fingerprint -> list of dep node indices.
  pub fn fingerprint_map(&self) -> &HashMap<Fingerprint, Vec<DepNodeIndex>> {
    self.fingerprint_map.get_or_init(|| {
      let mut map: HashMap<Fingerprint, Vec<DepNodeIndex>> = HashMap::new();
      for (i, node) in self.serialized.dep_graph.nodes.iter().enumerate() {
        map.entry(node.name()).or_default().push(i as DepNodeIndex);
      }
      map
    })
  }

  /// Find a DerivedQuery node by name + key fingerprint.
  pub fn find_derived_query(
    &self,
    name: Fingerprint,
    key: Fingerprint,
  ) -> Option<(DepNodeIndex, &DepNode)> {
    let indices = self.fingerprint_map().get(&name)?;
    indices.iter().find_map(|&idx| {
      let node = &self.serialized.dep_graph.nodes[idx as usize];
      if let DepNode::DerivedQuery { key: k, .. } = node {
        if k == &key {
          return Some((idx, node));
        }
      }
      None
    })
  }
}
