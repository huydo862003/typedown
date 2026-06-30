//! Dependency graph types for cache persistence.

use rustc_stable_hash::{FromStableHash, SipHasher128Hash};

use super::StableHasher;

/// A stable 128-bit hash value, used for both query identity and result change detection.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Fingerprint(pub [u8; 16]);

impl FromStableHash for Fingerprint {
  type Hash = SipHasher128Hash;

  fn from(hash: SipHasher128Hash) -> Self {
    let [lo, hi] = hash.0;
    let mut bytes = [0u8; 16];
    bytes[..8].copy_from_slice(&lo.to_le_bytes());
    bytes[8..].copy_from_slice(&hi.to_le_bytes());
    Fingerprint(bytes)
  }
}

impl Fingerprint {
  pub fn from_hasher(hasher: StableHasher) -> Self {
    hasher.finish()
  }
}
