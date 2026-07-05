//! A trait for session-independent & architecture-independent hashing
//! based on `rustc`: https://github.com/rust-lang/rust/blob/63f05e3635171e7ac3f9ca78bad6c71052cda5a3/compiler/rustc_data_structures/src/stable_hash.rs#L76-L78

use rustc_stable_hash::StableSipHasher128;
use std::hash::{Hash, Hasher};

use super::ord::StableOrd;

use crate::QueryDatabase;
use typedown_types::either::Either;

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
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher);
}

/// Hasher state to thread through multiple fields
pub type StableHasher = StableSipHasher128; // Same as what rustc uses

impl StableHash for i8 {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, _db: &DB, hasher: &mut StableHasher) {
    hasher.write_i8(*self);
  }
}
impl StableHash for i16 {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, _db: &DB, hasher: &mut StableHasher) {
    hasher.write_i16(*self);
  }
}
impl StableHash for i32 {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, _db: &DB, hasher: &mut StableHasher) {
    hasher.write_i32(*self);
  }
}
impl StableHash for i64 {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, _db: &DB, hasher: &mut StableHasher) {
    hasher.write_i64(*self);
  }
}
impl StableHash for i128 {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, _db: &DB, hasher: &mut StableHasher) {
    hasher.write_i128(*self);
  }
}
impl StableHash for isize {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, _db: &DB, hasher: &mut StableHasher) {
    hasher.write_isize(*self);
  }
}

impl StableHash for u8 {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, _db: &DB, hasher: &mut StableHasher) {
    hasher.write_u8(*self);
  }
}
impl StableHash for u16 {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, _db: &DB, hasher: &mut StableHasher) {
    hasher.write_u16(*self);
  }
}
impl StableHash for u32 {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, _db: &DB, hasher: &mut StableHasher) {
    hasher.write_u32(*self);
  }
}
impl StableHash for u64 {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, _db: &DB, hasher: &mut StableHasher) {
    hasher.write_u64(*self);
  }
}
impl StableHash for u128 {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, _db: &DB, hasher: &mut StableHasher) {
    hasher.write_u128(*self);
  }
}
impl StableHash for usize {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, _db: &DB, hasher: &mut StableHasher) {
    hasher.write_usize(*self);
  }
}

impl StableHash for bool {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, _db: &DB, hasher: &mut StableHasher) {
    hasher.write_u8(*self as u8);
  }
}
impl StableHash for char {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, _db: &DB, hasher: &mut StableHasher) {
    hasher.write_u32(*self as u32);
  }
}
impl StableHash for () {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, _db: &DB, _hasher: &mut StableHasher) {}
}

impl<T: StableHash> StableHash for [T] {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    self.len().stable_hash(db, hasher);
    for item in self {
      item.stable_hash(db, hasher);
    }
  }
}
impl<T: StableHash> StableHash for &[T] {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    (*self).stable_hash(db, hasher);
  }
}
impl<T: StableHash> StableHash for Vec<T> {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    self[..].stable_hash(db, hasher);
  }
}

impl StableHash for str {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    self.as_bytes().stable_hash(db, hasher);
  }
}
impl StableHash for &str {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    (*self).stable_hash(db, hasher);
  }
}
impl StableHash for String {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    self[..].stable_hash(db, hasher);
  }
}
impl StableHash for std::ffi::OsStr {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    self.as_encoded_bytes().stable_hash(db, hasher);
  }
}
impl StableHash for std::path::Path {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    self.as_os_str().stable_hash(db, hasher);
  }
}
impl StableHash for std::path::PathBuf {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    self.as_path().stable_hash(db, hasher);
  }
}

impl StableHash for &std::path::Path {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    (*self).stable_hash(db, hasher);
  }
}
impl StableHash for f32 {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    self.to_bits().stable_hash(db, hasher);
  }
}
impl StableHash for f64 {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    self.to_bits().stable_hash(db, hasher);
  }
}

impl StableHash for std::cmp::Ordering {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    (*self as i8).stable_hash(db, hasher);
  }
}

impl<T> StableHash for std::marker::PhantomData<T> {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, _db: &DB, _hasher: &mut StableHasher) {}
}

impl<T: StableHash + ?Sized> StableHash for Box<T> {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    (**self).stable_hash(db, hasher);
  }
}
impl<T: StableHash + ?Sized> StableHash for std::rc::Rc<T> {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    (**self).stable_hash(db, hasher);
  }
}
impl<T: StableHash + ?Sized> StableHash for std::sync::Arc<T> {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    (**self).stable_hash(db, hasher);
  }
}

impl<T1: StableHash, T2: StableHash> StableHash for Result<T1, T2> {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    // TIL: You can access the discriminant of an enum this way
    std::mem::discriminant(self).stable_hash(db, hasher);
    match self {
      Ok(val) => val.stable_hash(db, hasher),
      Err(val) => val.stable_hash(db, hasher),
    }
  }
}
impl<T> StableHash for std::mem::Discriminant<T> {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, _db: &DB, hasher: &mut StableHasher) {
    Hash::hash(self, hasher);
  }
}

impl<K: StableHash + StableOrd, V: StableHash> StableHash for std::collections::BTreeMap<K, V> {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    self.len().stable_hash(db, hasher);
    for entry in self.iter() {
      entry.stable_hash(db, hasher);
    }
  }
}
impl<K: StableHash + StableOrd> StableHash for std::collections::BTreeSet<K> {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    self.len().stable_hash(db, hasher);
    for entry in self.iter() {
      entry.stable_hash(db, hasher);
    }
  }
}

impl<T: StableHash> StableHash for Option<T> {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    match self {
      None => hasher.write_u8(0),
      Some(value) => {
        hasher.write_u8(1);
        value.stable_hash(db, hasher);
      }
    }
  }
}
impl<T: StableHash> StableHash for &T {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    (*self).stable_hash(db, hasher);
  }
}

impl<T: StableHash> StableHash for (T,) {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    self.0.stable_hash(db, hasher);
  }
}
impl<T1: StableHash, T2: StableHash> StableHash for (T1, T2) {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    self.0.stable_hash(db, hasher);
    self.1.stable_hash(db, hasher);
  }
}
impl<T1: StableHash, T2: StableHash, T3: StableHash> StableHash for (T1, T2, T3) {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    self.0.stable_hash(db, hasher);
    self.1.stable_hash(db, hasher);
    self.2.stable_hash(db, hasher);
  }
}
impl<T1: StableHash, T2: StableHash, T3: StableHash, T4: StableHash> StableHash
  for (T1, T2, T3, T4)
{
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    self.0.stable_hash(db, hasher);
    self.1.stable_hash(db, hasher);
    self.2.stable_hash(db, hasher);
    self.3.stable_hash(db, hasher);
  }
}

impl<K: StableHash + StableOrd, V: StableHash> StableHash for std::collections::HashMap<K, V> {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    self.len().stable_hash(db, hasher);
    let mut entries: Vec<(&K, &V)> = self.iter().collect();
    entries.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
    for (key, value) in entries {
      key.stable_hash(db, hasher);
      value.stable_hash(db, hasher);
    }
  }
}

impl<K: StableHash + StableOrd> StableHash for std::collections::HashSet<K> {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    self.len().stable_hash(db, hasher);
    let mut entries: Vec<&K> = self.iter().collect();
    entries.sort_by(|k1, k2| k1.cmp(k2));
    for key in entries {
      key.stable_hash(db, hasher);
    }
  }
}

impl StableHash for std::time::SystemTime {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    match self.duration_since(std::time::UNIX_EPOCH) {
      Ok(dur) => {
        0u8.stable_hash(db, hasher);
        dur.as_secs().stable_hash(db, hasher);
        dur.subsec_nanos().stable_hash(db, hasher);
      }
      Err(err) => {
        1u8.stable_hash(db, hasher);
        err.duration().as_secs().stable_hash(db, hasher);
        err.duration().subsec_nanos().stable_hash(db, hasher);
      }
    }
  }
}

impl<L: StableHash, R: StableHash> StableHash for Either<L, R> {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    std::mem::discriminant(self).stable_hash(db, hasher);
    match self {
      Either::Left(val) => val.stable_hash(db, hasher),
      Either::Right(val) => val.stable_hash(db, hasher),
    }
  }
}

// typedown-types impls

use typedown_types::diagnostic::Diagnostic;
use typedown_types::syntax_kind::SyntaxKind;

impl StableHash for SyntaxKind {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, _db: &DB, hasher: &mut StableHasher) {
    std::hash::Hasher::write_u16(hasher, *self as u16);
  }
}

impl StableHash for Diagnostic {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    std::mem::discriminant(self).stable_hash(db, hasher);
    match self {
      Diagnostic::UnexpectedEof {
        expected,
        start_offset,
        end_offset,
      } => {
        expected.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::UnexpectedChar {
        expected,
        encountered,
        start_offset,
        end_offset,
      } => {
        expected.stable_hash(db, hasher);
        encountered.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::InvalidChar {
        encountered,
        start_offset,
        end_offset,
      } => {
        encountered.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::InconsistentIndentation {
        expected,
        encountered,
        start_offset,
        end_offset,
      } => {
        expected.stable_hash(db, hasher);
        encountered.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::UnmatchedDedent {
        indent,
        start_offset,
        end_offset,
      } => {
        indent.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::MissingFrontmatterMarker { offset } => {
        offset.stable_hash(db, hasher);
      }
      Diagnostic::MissingSyntaxNode {
        expected,
        start_offset,
        end_offset,
      } => {
        expected.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::MissingExpectMdPrefix {
        expected_prefix,
        start_offset,
        end_offset,
      } => {
        expected_prefix.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::TableColumnCountMismatch {
        expected,
        found,
        start_offset,
        end_offset,
      } => {
        expected.stable_hash(db, hasher);
        found.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::InsufficientBlockIndent {
        expected_more_than,
        found,
        start_offset,
        end_offset,
      } => {
        expected_more_than.stable_hash(db, hasher);
        found.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::MissingVaultConfig { root_dir } => {
        root_dir.stable_hash(db, hasher);
      }
      Diagnostic::VaultConfigReadError { path, message } => {
        path.stable_hash(db, hasher);
        message.stable_hash(db, hasher);
      }
      Diagnostic::VaultConfigParseError {
        path,
        message,
        start_offset,
        end_offset,
      } => {
        path.stable_hash(db, hasher);
        message.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::VaultConfigEmpty { path } => {
        path.stable_hash(db, hasher);
      }
      Diagnostic::VaultConfigMissingField {
        path,
        field,
        start_offset,
        end_offset,
      } => {
        path.stable_hash(db, hasher);
        field.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::VaultConfigUnknownField {
        path,
        field,
        start_offset,
        end_offset,
      } => {
        path.stable_hash(db, hasher);
        field.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::UnresolvedSchema {
        name,
        start_offset,
        end_offset,
      } => {
        name.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::WrongTypeArgCount { expected, got } => {
        expected.stable_hash(db, hasher);
        got.stable_hash(db, hasher);
      }
      Diagnostic::WrongArgCount {
        expected,
        got,
        start_offset,
        end_offset,
      } => {
        expected.stable_hash(db, hasher);
        got.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::ArgTypeMismatch {
        expected,
        start_offset,
        end_offset,
      } => {
        expected.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::FieldTypeMismatch {
        field,
        expected,
        start_offset,
        end_offset,
      } => {
        field.stable_hash(db, hasher);
        expected.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::IndexTypeMismatch {
        expected,
        start_offset,
        end_offset,
      } => {
        expected.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::TagTypeMismatch {
        expected,
        start_offset,
        end_offset,
      } => {
        expected.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::OperandTypeMismatch {
        op,
        expected,
        start_offset,
        end_offset,
      } => {
        op.stable_hash(db, hasher);
        expected.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::MissingRequiredField {
        field,
        start_offset,
        end_offset,
      } => {
        field.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::ElementTypeMismatch {
        expected,
        start_offset,
        end_offset,
      } => {
        expected.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::DuplicateKey {
        key,
        start_offset,
        end_offset,
      } => {
        key.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::UnresolvedFileRef {
        path,
        start_offset,
        end_offset,
      } => {
        path.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::UnknownField {
        field,
        on_type,
        start_offset,
        end_offset,
      } => {
        field.stable_hash(db, hasher);
        on_type.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
      Diagnostic::IndexOutOfBounds {
        index,
        length,
        start_offset,
        end_offset,
      } => {
        index.stable_hash(db, hasher);
        length.stable_hash(db, hasher);
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
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
        start_offset.stable_hash(db, hasher);
        end_offset.stable_hash(db, hasher);
      }
    }
  }
}
