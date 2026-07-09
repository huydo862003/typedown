use std::any::Any;

use super::storage::QueryStorage;
use crate::SerializeContext;
use crate::persist::serialized::SerializedQueryStorage;
use crate::persist::serialized::dep_graph::{self as dep_graph_format, DepGraph};
use crate::persist::serialized::interned_blobs::{
  self as interned_blobs_format, InternedBlobs, NodeRecord,
};
use crate::persist::serialized::query_cache::QueryCache;

pub trait QueryDatabase: Any {
  #[doc(hidden)]
  unsafe fn storage(&self) -> &QueryStorage;

  #[doc(hidden)]
  unsafe fn storage_mut(&mut self) -> &mut QueryStorage;
}

/// TIL:
/// We can seal some methods of a trait T by adding a private module and a private trait local here
/// Then we can require the sealed trait to implement the the private trait local here
/// Then we add a blanket implementation (see the one below)
mod sealed {
  use crate::QueryDatabase;

  pub trait Sealed {}
  impl<T: QueryDatabase> Sealed for T {}
}

impl<T: QueryDatabase> SerializableQueryDatabase for T {}

/// Extension of QueryDatabase that supports serialization.
/// All methods have fixed implementations and cannot be overridden.
pub trait SerializableQueryDatabase: QueryDatabase + sealed::Sealed {
  /// Serialize the current query storage into the serialized formats.
  fn dump(&self) -> SerializedQueryStorage
  where
    Self: Sized,
  {
    let storage = unsafe { self.storage() };
    let mut ctx = SerializeContext::new(self);

    // Serialize all ingredients
    for entry in storage.ingredients.iter() {
      let ingredient = &entry.ingredient;
      for entry_id in ingredient.entry_ids().collect::<Vec<_>>() {
        ingredient.serialize(&mut ctx, entry_id);
      }
    }

    // Finalize
    let (nodes, query_cache_mmap, intern_blobs) = ctx.finalize();

    // Build DepGraph
    let total_edge_count = nodes.iter().map(|n| n.edges().len() as u64).sum();
    let dep_graph = DepGraph {
      header: dep_graph_format::FileHeader::new(
        storage.revision.load(std::sync::atomic::Ordering::Acquire) as u64,
      ),
      footer: dep_graph_format::FileFooter {
        total_node_count: nodes.len() as u64,
        total_edge_count,
      },
      nodes,
    };

    // Build QueryCache from mmap
    let query_cache = unsafe {
      QueryCache::from_raw(query_cache_mmap.as_ptr(), query_cache_mmap.len())
        .expect("Failed to construct QueryCache from serialized data")
    };

    // Build InternedBlobs
    let records: Vec<NodeRecord> = intern_blobs
      .into_iter()
      .map(|blob| {
        let (record, _) = NodeRecord::from_bytes(&blob);
        record
      })
      .collect();
    let total_byte_size: u64 = records.iter().map(|r| r.to_bytes().len() as u64).sum();
    let interned_blobs = InternedBlobs {
      header: interned_blobs_format::FileHeader::new(),
      footer: interned_blobs_format::FileFooter {
        total_node_count: records.len() as u64,
        total_byte_size,
      },
      records,
    };

    SerializedQueryStorage {
      dep_graph,
      query_cache,
      interned_blobs,
    }
  }

  /// Load query storage from the serialized formats.
  fn load(&self, _serialized: &SerializedQueryStorage)
  where
    Self: Sized,
  {
    todo!()
  }
}
