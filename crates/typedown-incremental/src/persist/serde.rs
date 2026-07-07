use crate::persist::serialized::dep_graph::DepNode;
use crate::{Decoder, Encoder};

/// Context for serializing ingredients during dump.
/// Accumulates dep graph nodes and streams query result blobs.
pub struct SerializeContext<'a> {
  pub encoder: &'a mut dyn Encoder,
  pub dep_nodes: Vec<DepNode>,
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
