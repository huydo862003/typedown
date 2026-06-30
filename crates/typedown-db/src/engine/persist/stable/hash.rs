//! A trait for session-independent & architecture-independent hashing
//! based on `rustc`: https://github.com/rust-lang/rust/blob/63f05e3635171e7ac3f9ca78bad6c71052cda5a3/compiler/rustc_data_structures/src/stable_hash.rs#L76-L78

use siphasher::sip::SipHasher13;

use crate::{DepId, DepPathHash, Span};

/// The following is the original rustc comment: '''
/// Note that `StableHash` imposes rather more strict requirements than usual
/// hash functions:
///
/// - Stable hashes are sometimes used as identifiers. Therefore they must
///   conform to the corresponding `PartialEq` implementations:
///
///     - `x == y` implies `stable_hash(x) == stable_hash(y)`, and
///     - `x != y` implies `stable_hash(x) != stable_hash(y)`.
///
///   That second condition is usually not required for hash functions
///   (e.g. `Hash`). In practice this means that `stable_hash` must feed any
///   information into the hasher that a `PartialEq` comparison takes into
///   account. See [#49300](https://github.com/rust-lang/rust/issues/49300)
///   for an example where violating this invariant has caused trouble in the
///   past.
///
/// - `stable_hash()` must be independent of the current
///    compilation session. E.g. they must not hash memory addresses or other
///    things that are "randomly" assigned per compilation session.
///
/// - `stable_hash()` must be independent of the host architecture. The
///   `StableHasher` takes care of endianness and `isize`/`usize` platform
///   differences.
/// '''
pub trait StableHash {
  fn stable_hash<H: StableHashCtx>(&self, hcx: &mut H, hasher: &mut StableHasher);
}

/// Hasher state to thread through multiple fields
pub type StableHasher = SipHasher13; // Same as what rustc uses

/// We need StableHashCtx to reliably hash the node/symbol id that correspond to a graph dep node
/// As graph dep node's id is session-dependent and prone to shifting
pub trait StableHashCtx {
  /// Allow hashing a graph dep node's span
  /// As span is prone to shifting
  fn stable_hash_span(&mut self, span: Span, hasher: &mut StableHasher);

  /// Compute a stable hash for a graph dep node id
  /// That is a symbol/node id
  fn dep_path_hash(&self, dep_id: DepId) -> DepPathHash;
}
