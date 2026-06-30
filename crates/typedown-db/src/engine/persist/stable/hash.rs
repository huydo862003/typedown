//! A trait for session-independent & architecture-independent hashing
//! based on `rustc`: https://github.com/rust-lang/rust/blob/63f05e3635171e7ac3f9ca78bad6c71052cda5a3/compiler/rustc_data_structures/src/stable_hash.rs#L76-L78

use rustc_stable_hash::StableSipHasher128;
use std::hash::{Hash, Hasher};

use super::ord::StableOrd;

use crate::types::FileHandle;
use crate::{DepId, DepPathHash, QueryDatabase, Span};
use typedown_types::{diagnostic::Diagnostic, syntax_kind::SyntaxKind};

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

/// Stable hash for types from typedown-syntax
impl StableHash for SyntaxKind {
  fn stable_hash<Hcx: StableHashCtx>(&self, _hcx: &mut Hcx, hasher: &mut StableHasher) {
    hasher.write_u16(*self as u16);
  }
}

impl StableHash for Diagnostic {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    std::mem::discriminant(self).stable_hash(hcx, hasher);
    match self {
      Diagnostic::UnexpectedEof {
        expected,
        start_offset,
        end_offset,
      } => {
        expected.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::UnexpectedChar {
        expected,
        encountered,
        start_offset,
        end_offset,
      } => {
        expected.stable_hash(hcx, hasher);
        encountered.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::InvalidChar {
        encountered,
        start_offset,
        end_offset,
      } => {
        encountered.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::InconsistentIndentation {
        expected,
        encountered,
        start_offset,
        end_offset,
      } => {
        expected.stable_hash(hcx, hasher);
        encountered.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::UnmatchedDedent {
        indent,
        start_offset,
        end_offset,
      } => {
        indent.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::MissingFrontmatterMarker { offset } => {
        offset.stable_hash(hcx, hasher);
      }
      Diagnostic::MissingSyntaxNode {
        expected,
        start_offset,
        end_offset,
      } => {
        expected.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::MissingExpectMdPrefix {
        expected_prefix,
        start_offset,
        end_offset,
      } => {
        expected_prefix.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::TableColumnCountMismatch {
        expected,
        found,
        start_offset,
        end_offset,
      } => {
        expected.stable_hash(hcx, hasher);
        found.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::InsufficientBlockIndent {
        expected_more_than,
        found,
        start_offset,
        end_offset,
      } => {
        expected_more_than.stable_hash(hcx, hasher);
        found.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::MissingVaultConfig { root_dir } => {
        root_dir.stable_hash(hcx, hasher);
      }
      Diagnostic::VaultConfigReadError { path, message } => {
        path.stable_hash(hcx, hasher);
        message.stable_hash(hcx, hasher);
      }
      Diagnostic::VaultConfigParseError {
        path,
        message,
        start_offset,
        end_offset,
      } => {
        path.stable_hash(hcx, hasher);
        message.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::VaultConfigEmpty { path } => {
        path.stable_hash(hcx, hasher);
      }
      Diagnostic::VaultConfigMissingField {
        path,
        field,
        start_offset,
        end_offset,
      } => {
        path.stable_hash(hcx, hasher);
        field.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::VaultConfigUnknownField {
        path,
        field,
        start_offset,
        end_offset,
      } => {
        path.stable_hash(hcx, hasher);
        field.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::UnresolvedSchema {
        name,
        start_offset,
        end_offset,
      } => {
        name.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::WrongTypeArgCount { expected, got } => {
        expected.stable_hash(hcx, hasher);
        got.stable_hash(hcx, hasher);
      }
      Diagnostic::WrongArgCount {
        expected,
        got,
        start_offset,
        end_offset,
      } => {
        expected.stable_hash(hcx, hasher);
        got.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::ArgTypeMismatch {
        expected,
        start_offset,
        end_offset,
      } => {
        expected.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::FieldTypeMismatch {
        field,
        expected,
        start_offset,
        end_offset,
      } => {
        field.stable_hash(hcx, hasher);
        expected.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::IndexTypeMismatch {
        expected,
        start_offset,
        end_offset,
      } => {
        expected.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::TagTypeMismatch {
        expected,
        start_offset,
        end_offset,
      } => {
        expected.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::OperandTypeMismatch {
        op,
        expected,
        start_offset,
        end_offset,
      } => {
        op.stable_hash(hcx, hasher);
        expected.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::MissingRequiredField {
        field,
        start_offset,
        end_offset,
      } => {
        field.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::ElementTypeMismatch {
        expected,
        start_offset,
        end_offset,
      } => {
        expected.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::DuplicateKey {
        key,
        start_offset,
        end_offset,
      } => {
        key.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::UnresolvedFileRef {
        path,
        start_offset,
        end_offset,
      } => {
        path.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::UnknownField {
        field,
        on_type,
        start_offset,
        end_offset,
      } => {
        field.stable_hash(hcx, hasher);
        on_type.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      Diagnostic::IndexOutOfBounds {
        index,
        length,
        start_offset,
        end_offset,
      } => {
        index.stable_hash(hcx, hasher);
        length.stable_hash(hcx, hasher);
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
      // Variants with only start_offset and end_offset: discriminant already distinguishes them
      Diagnostic::UnterminatedString {
        start_offset,
        end_offset,
      }
      | Diagnostic::UnterminatedInterpolation {
        start_offset,
        end_offset,
      }
      | Diagnostic::UnterminatedCodeBlock {
        start_offset,
        end_offset,
      }
      | Diagnostic::UnterminatedInlineCode {
        start_offset,
        end_offset,
      }
      | Diagnostic::UnterminatedMathBlock {
        start_offset,
        end_offset,
      }
      | Diagnostic::UnterminatedInlineMath {
        start_offset,
        end_offset,
      }
      | Diagnostic::MissingCodeBlockNewline {
        start_offset,
        end_offset,
      }
      | Diagnostic::MissingMathBlockNewline {
        start_offset,
        end_offset,
      }
      | Diagnostic::InvalidUtf8 {
        start_offset,
        end_offset,
      }
      | Diagnostic::MixedIndentation {
        start_offset,
        end_offset,
      }
      | Diagnostic::MissingExponentDigits {
        start_offset,
        end_offset,
      }
      | Diagnostic::UnexpectedTokensOnFrontmatterMarkerLine {
        start_offset,
        end_offset,
      }
      | Diagnostic::MissingMarkdownHeadingHash {
        start_offset,
        end_offset,
      }
      | Diagnostic::MissingRequiredSpacesBetweenHashAndHeading {
        start_offset,
        end_offset,
      }
      | Diagnostic::UnclosedLink {
        start_offset,
        end_offset,
      }
      | Diagnostic::UnclosedBold {
        start_offset,
        end_offset,
      }
      | Diagnostic::UnclosedItalic {
        start_offset,
        end_offset,
      }
      | Diagnostic::UnclosedStrikethrough {
        start_offset,
        end_offset,
      }
      | Diagnostic::UnclosedBoldItalic {
        start_offset,
        end_offset,
      }
      | Diagnostic::MismatchedItalicDelimiter {
        start_offset,
        end_offset,
      }
      | Diagnostic::MissingTableSeparatorRow {
        start_offset,
        end_offset,
      }
      | Diagnostic::MissingSchemaField {
        start_offset,
        end_offset,
      }
      | Diagnostic::NotCallable {
        start_offset,
        end_offset,
      }
      | Diagnostic::NotIndexable {
        start_offset,
        end_offset,
      } => {
        start_offset.stable_hash(hcx, hasher);
        end_offset.stable_hash(hcx, hasher);
      }
    }
  }
}

impl StableHash for FileHandle {
  fn stable_hash<Hcx: StableHashCtx>(&self, hcx: &mut Hcx, hasher: &mut StableHasher) {
    std::mem::discriminant(self).stable_hash(hcx, hasher);
    match self {
      FileHandle::Path(path, mtime) => {
        path.stable_hash(hcx, hasher);
        // Convert mtime to (secs, nanos) since UNIX_EPOCH for a stable byte representation
        let duration = mtime
          .duration_since(std::time::UNIX_EPOCH)
          .unwrap_or_default();
        duration.as_secs().stable_hash(hcx, hasher);
        duration.subsec_nanos().stable_hash(hcx, hasher);
      }
      FileHandle::Content(content) => {
        content.stable_hash(hcx, hasher);
      }
    }
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

  /// Access the database
  fn db(&self) -> &dyn QueryDatabase;
}
