//! Serialized formats for cross-session persistence.
//!
//! Three files are persisted:
//! - `dep-graph.bin`: The dependency graph (nodes, edges, fingerprints). No query result data.
//! - `query-cache.bin`: Cached query results, accessed lazily by offset.
//! - `interned-blobs.bin`: Deduplicated blobs (e.g. green tree nodes).

pub mod dep_graph;
pub mod interned_blobs;
pub mod query_cache;

use dep_graph::DepGraph;
use interned_blobs::InternedBlobs;
use query_cache::QueryCache;

/// The serialized form of the entire query storage.
/// Produced by `dump`, consumed by `load`.
pub struct SerializedQueryStorage {
  pub dep_graph: DepGraph,
  pub query_cache: QueryCache,
  pub interned_blobs: InternedBlobs,
}
