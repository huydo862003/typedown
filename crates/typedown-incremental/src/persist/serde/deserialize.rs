use std::collections::HashMap;

use crate::Fingerprint;
use crate::persist::serialized::dep_graph::{DepNode, DepNodeIndex};
use crate::persist::serialized::query_cache::QueryCache;

/// Context for deserializing ingredients during load.
/// Provides access to the previously serialized data.
pub struct DeserializeContext<'a> {
  /// Ingredient name fingerprint -> list of (node_index, DepNode)
  pub nodes_by_name: HashMap<Fingerprint, Vec<(DepNodeIndex, &'a DepNode)>>,
  pub query_cache: &'a QueryCache,
}
