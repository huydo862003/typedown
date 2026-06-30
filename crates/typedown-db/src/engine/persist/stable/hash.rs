//! A trait for session-independent & architecture-independent hashing
//! based on `rustc`: https://github.com/rust-lang/rust/blob/63f05e3635171e7ac3f9ca78bad6c71052cda5a3/compiler/rustc_data_structures/src/stable_hash.rs#L76-L78

use rustc_stable_hash::StableSipHasher128;
use std::hash::{Hash, Hasher};

use super::ord::StableOrd;

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
pub type StableHasher = StableSipHasher128; // Same as what rustc uses

impl StableHash for i8 {
  fn stable_hash<Hcx: StableHashCtx>(&self, _hcx: &mut Hcx, hasher: &mut StableHasher) {
    hasher.write_i8(*self);
  }
}
impl StableHash for i16 {
  fn stable_hash<Hcx: StableHashCtx>(&self, _hcx: &mut Hcx, hasher: &mut StableHasher) {
    hasher.write_i16(*self);
  }
}
impl StableHash for i32 {
  fn stable_hash<Hcx: StableHashCtx>(&self, _hcx: &mut Hcx, hasher: &mut StableHasher) {
    hasher.write_i32(*self);
  }
}
impl StableHash for i64 {
  fn stable_hash<Hcx: StableHashCtx>(&self, _hcx: &mut Hcx, hasher: &mut StableHasher) {
    hasher.write_i64(*self);
  }
}
impl StableHash for i128 {
  fn stable_hash<Hcx: StableHashCtx>(&self, _hcx: &mut Hcx, hasher: &mut StableHasher) {
    hasher.write_i128(*self);
  }
}
impl StableHash for isize {
  fn stable_hash<Hcx: StableHashCtx>(&self, _hcx: &mut Hcx, hasher: &mut StableHasher) {
    hasher.write_isize(*self);
  }
}

impl StableHash for u8 {
  fn stable_hash<Hcx: StableHashCtx>(&self, _hcx: &mut Hcx, hasher: &mut StableHasher) {
    hasher.write_u8(*self);
  }
}
impl StableHash for u16 {
  fn stable_hash<Hcx: StableHashCtx>(&self, _hcx: &mut Hcx, hasher: &mut StableHasher) {
    hasher.write_u16(*self);
  }
}
impl StableHash for u32 {
  fn stable_hash<Hcx: StableHashCtx>(&self, _hcx: &mut Hcx, hasher: &mut StableHasher) {
    hasher.write_u32(*self);
  }
}
impl StableHash for u64 {
  fn stable_hash<Hcx: StableHashCtx>(&self, _hcx: &mut Hcx, hasher: &mut StableHasher) {
    hasher.write_u64(*self);
  }
}
impl StableHash for u128 {
  fn stable_hash<Hcx: StableHashCtx>(&self, _hcx: &mut Hcx, hasher: &mut StableHasher) {
    hasher.write_u128(*self);
  }
}
impl StableHash for usize {
  fn stable_hash<Hcx: StableHashCtx>(&self, _hcx: &mut Hcx, hasher: &mut StableHasher) {
    hasher.write_usize(*self);
  }
}

impl StableHash for bool {
  fn stable_hash<Hcx: StableHashCtx>(&self, _hcx: &mut Hcx, hasher: &mut StableHasher) {
    hasher.write_u8(*self as u8);
  }
}
impl StableHash for char {
  fn stable_hash<Hcx: StableHashCtx>(&self, _hcx: &mut Hcx, hasher: &mut StableHasher) {
    hasher.write_u32(*self as u32);
  }
}
impl StableHash for () {
  fn stable_hash<Hcx: StableHashCtx>(&self, _hcx: &mut Hcx, _hasher: &mut StableHasher) {}
}

impl<T: StableHash> StableHash for [T] {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    self.len().stable_hash(hcx, hasher);
    for item in self {
      item.stable_hash(hcx, hasher);
    }
  }
}
impl<T: StableHash> StableHash for &[T] {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    (*self).stable_hash(hcx, hasher);
  }
}
impl<T: StableHash> StableHash for Vec<T> {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    self[..].stable_hash(hcx, hasher);
  }
}

impl StableHash for str {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    self.as_bytes().stable_hash(hcx, hasher);
  }
}
impl StableHash for &str {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    (*self).stable_hash(hcx, hasher);
  }
}
impl StableHash for String {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    self[..].stable_hash(hcx, hasher);
  }
}
impl StableHash for std::ffi::OsStr {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    self.as_encoded_bytes().stable_hash(hcx, hasher);
  }
}
impl StableHash for std::path::Path {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    self.as_os_str().stable_hash(hcx, hasher);
  }
}
impl StableHash for std::path::PathBuf {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    self.as_path().stable_hash(hcx, hasher);
  }
}

impl StableHash for &std::path::Path {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    (*self).stable_hash(hcx, hasher);
  }
}
impl StableHash for f32 {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    self.to_bits().stable_hash(hcx, hasher);
  }
}
impl StableHash for f64 {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    self.to_bits().stable_hash(hcx, hasher);
  }
}

impl StableHash for std::cmp::Ordering {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    (*self as i8).stable_hash(hcx, hasher);
  }
}

impl<T> StableHash for std::marker::PhantomData<T> {
  fn stable_hash<Hcx: StableHashCtx>(&self, _hcx: &mut Hcx, _hasher: &mut StableHasher) {}
}

impl<T: StableHash> StableHash for Box<T> {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    (**self).stable_hash(hcx, hasher);
  }
}
impl<T: StableHash + ?Sized> StableHash for std::rc::Rc<T> {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    (**self).stable_hash(hcx, hasher);
  }
}
impl<T: StableHash + ?Sized> StableHash for std::sync::Arc<T> {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    (**self).stable_hash(hcx, hasher);
  }
}

impl<T1: StableHash, T2: StableHash> StableHash for Result<T1, T2> {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    // TIL: You can access the discriminant of an enum this way
    std::mem::discriminant(self).stable_hash(hcx, hasher);
    match self {
      Ok(val) => val.stable_hash(hcx, hasher),
      Err(val) => val.stable_hash(hcx, hasher),
    }
  }
}
impl<T> StableHash for std::mem::Discriminant<T> {
  fn stable_hash<Hcx: StableHashCtx>(&self, _hcx: &mut Hcx, hasher: &mut StableHasher) {
    Hash::hash(self, hasher);
  }
}

impl<K: StableHash + StableOrd, V: StableHash> StableHash for std::collections::BTreeMap<K, V> {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    self.len().stable_hash(hcx, hasher);
    for entry in self.iter() {
      entry.stable_hash(hcx, hasher);
    }
  }
}
impl<K: StableHash + StableOrd> StableHash for std::collections::BTreeSet<K> {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    self.len().stable_hash(hcx, hasher);
    for entry in self.iter() {
      entry.stable_hash(hcx, hasher);
    }
  }
}

impl<T: StableHash> StableHash for Option<T> {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    match self {
      None => hasher.write_u8(0),
      Some(value) => {
        hasher.write_u8(1);
        value.stable_hash(hcx, hasher);
      }
    }
  }
}
impl<T: StableHash> StableHash for &T {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    (*self).stable_hash(hcx, hasher);
  }
}

impl<T: StableHash> StableHash for (T,) {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    self.0.stable_hash(hcx, hasher);
  }
}
impl<T1: StableHash, T2: StableHash> StableHash for (T1, T2) {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    self.0.stable_hash(hcx, hasher);
    self.1.stable_hash(hcx, hasher);
  }
}
impl<T1: StableHash, T2: StableHash, T3: StableHash> StableHash for (T1, T2, T3) {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    self.0.stable_hash(hcx, hasher);
    self.1.stable_hash(hcx, hasher);
    self.2.stable_hash(hcx, hasher);
  }
}
impl<T1: StableHash, T2: StableHash, T3: StableHash, T4: StableHash> StableHash
  for (T1, T2, T3, T4)
{
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    self.0.stable_hash(hcx, hasher);
    self.1.stable_hash(hcx, hasher);
    self.2.stable_hash(hcx, hasher);
    self.3.stable_hash(hcx, hasher);
  }
}

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
