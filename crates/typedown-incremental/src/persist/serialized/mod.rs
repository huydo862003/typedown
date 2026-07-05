//! Serialized formats for cross-session persistence.
//!
//! Three files are persisted:
//! - `dep-graph.bin`: The dependency graph (nodes, edges, fingerprints). No query result data.
//! - `query-cache.bin`: Cached query results, accessed lazily by offset.
//! - `interned-nodes.bin`: Deduplicated green tree nodes (tokens and inner nodes).

pub mod dep_graph;
pub mod interned_nodes;
pub mod query_cache;
