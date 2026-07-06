use std::collections::HashMap;
use std::hash::Hash;
use std::path::PathBuf;

use typedown_types::either::Either;

use crate::QueryDatabase;

// Object-safe encoder trait
pub trait Encoder {
  fn db(&self) -> &dyn QueryDatabase;
  fn emit_raw(&mut self, bytes: &[u8]);
  fn intern_blob(&mut self, blob: Vec<u8>, hint: Option<usize>) -> u32;

  fn emit_u8(&mut self, v: u8) {
    self.emit_raw(&[v]);
  }
  fn emit_u16(&mut self, v: u16) {
    self.emit_raw(&v.to_le_bytes());
  }
  fn emit_u32(&mut self, v: u32) {
    self.emit_raw(&v.to_le_bytes());
  }
  fn emit_u64(&mut self, v: u64) {
    self.emit_raw(&v.to_le_bytes());
  }
  fn emit_u128(&mut self, v: u128) {
    self.emit_raw(&v.to_le_bytes());
  }
  fn emit_usize(&mut self, v: usize) {
    self.emit_u64(v as u64);
  }
  fn emit_isize(&mut self, v: isize) {
    self.emit_i64(v as i64);
  }
  fn emit_i8(&mut self, v: i8) {
    self.emit_raw(&[v as u8]);
  }
  fn emit_i16(&mut self, v: i16) {
    self.emit_raw(&v.to_le_bytes());
  }
  fn emit_i32(&mut self, v: i32) {
    self.emit_raw(&v.to_le_bytes());
  }
  fn emit_i64(&mut self, v: i64) {
    self.emit_raw(&v.to_le_bytes());
  }
  fn emit_i128(&mut self, v: i128) {
    self.emit_raw(&v.to_le_bytes());
  }
  fn emit_f64(&mut self, v: f64) {
    self.emit_raw(&v.to_le_bytes());
  }
  fn emit_bool(&mut self, v: bool) {
    self.emit_u8(v as u8);
  }
  fn emit_char(&mut self, v: char) {
    self.emit_u32(v as u32);
  }
  fn emit_str(&mut self, v: &str) {
    self.emit_u32(v.len() as u32);
    self.emit_raw(v.as_bytes());
  }
  fn emit_bytes(&mut self, v: &[u8]) {
    self.emit_u32(v.len() as u32);
    self.emit_raw(v);
  }
}

// Object-safe decoder trait
pub trait Decoder {
  fn db(&self) -> &dyn QueryDatabase;
  fn read_raw(&mut self, buf: &mut [u8]);
  fn get_intern_blob(&self, index: u32) -> &[u8];

  fn read_u8(&mut self) -> u8 {
    let mut buf = [0u8; 1];
    self.read_raw(&mut buf);
    buf[0]
  }
  fn read_u16(&mut self) -> u16 {
    let mut buf = [0u8; 2];
    self.read_raw(&mut buf);
    u16::from_le_bytes(buf)
  }
  fn read_u32(&mut self) -> u32 {
    let mut buf = [0u8; 4];
    self.read_raw(&mut buf);
    u32::from_le_bytes(buf)
  }
  fn read_u64(&mut self) -> u64 {
    let mut buf = [0u8; 8];
    self.read_raw(&mut buf);
    u64::from_le_bytes(buf)
  }
  fn read_u128(&mut self) -> u128 {
    let mut buf = [0u8; 16];
    self.read_raw(&mut buf);
    u128::from_le_bytes(buf)
  }
  fn read_usize(&mut self) -> usize {
    self.read_u64() as usize
  }
  fn read_isize(&mut self) -> isize {
    self.read_i64() as isize
  }
  fn read_i8(&mut self) -> i8 {
    self.read_u8() as i8
  }
  fn read_i16(&mut self) -> i16 {
    let mut buf = [0u8; 2];
    self.read_raw(&mut buf);
    i16::from_le_bytes(buf)
  }
  fn read_i32(&mut self) -> i32 {
    let mut buf = [0u8; 4];
    self.read_raw(&mut buf);
    i32::from_le_bytes(buf)
  }
  fn read_i64(&mut self) -> i64 {
    let mut buf = [0u8; 8];
    self.read_raw(&mut buf);
    i64::from_le_bytes(buf)
  }
  fn read_i128(&mut self) -> i128 {
    let mut buf = [0u8; 16];
    self.read_raw(&mut buf);
    i128::from_le_bytes(buf)
  }
  fn read_f64(&mut self) -> f64 {
    let mut buf = [0u8; 8];
    self.read_raw(&mut buf);
    f64::from_le_bytes(buf)
  }
  fn read_bool(&mut self) -> bool {
    self.read_u8() != 0
  }
  fn read_char(&mut self) -> char {
    char::from_u32(self.read_u32()).unwrap()
  }
  fn read_str(&mut self) -> String {
    let len = self.read_u32() as usize;
    let mut buf = vec![0u8; len];
    self.read_raw(&mut buf);
    String::from_utf8(buf).unwrap()
  }
  fn read_bytes_owned(&mut self) -> Vec<u8> {
    let len = self.read_u32() as usize;
    let mut buf = vec![0u8; len];
    self.read_raw(&mut buf);
    buf
  }
}

// Encodable / Decodable traits
pub trait Encodable {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E);
}

pub trait Decodable: Sized {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self;
}

/* Primitive implementations */

// u8
impl Encodable for u8 {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_u8(*self);
  }
}
impl Decodable for u8 {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    decoder.read_u8()
  }
}

// u16
impl Encodable for u16 {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_u16(*self);
  }
}
impl Decodable for u16 {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    decoder.read_u16()
  }
}

// u32
impl Encodable for u32 {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_u32(*self);
  }
}
impl Decodable for u32 {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    decoder.read_u32()
  }
}

// u64
impl Encodable for u64 {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_u64(*self);
  }
}
impl Decodable for u64 {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    decoder.read_u64()
  }
}

// usize (encoded as u64)
impl Encodable for usize {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_usize(*self);
  }
}
impl Decodable for usize {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    decoder.read_usize()
  }
}

// u128
impl Encodable for u128 {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_u128(*self);
  }
}
impl Decodable for u128 {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    decoder.read_u128()
  }
}

// isize
impl Encodable for isize {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_isize(*self);
  }
}
impl Decodable for isize {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    decoder.read_isize()
  }
}

// i8
impl Encodable for i8 {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_i8(*self);
  }
}
impl Decodable for i8 {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    decoder.read_i8()
  }
}

// i16
impl Encodable for i16 {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_i16(*self);
  }
}
impl Decodable for i16 {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    decoder.read_i16()
  }
}

// i32
impl Encodable for i32 {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_i32(*self);
  }
}
impl Decodable for i32 {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    decoder.read_i32()
  }
}

// i64
impl Encodable for i64 {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_i64(*self);
  }
}
impl Decodable for i64 {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    decoder.read_i64()
  }
}

// i128
impl Encodable for i128 {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_i128(*self);
  }
}
impl Decodable for i128 {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    decoder.read_i128()
  }
}

// f64
impl Encodable for f64 {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_f64(*self);
  }
}
impl Decodable for f64 {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    decoder.read_f64()
  }
}

// bool
impl Encodable for bool {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_bool(*self);
  }
}
impl Decodable for bool {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    decoder.read_bool()
  }
}

// char
impl Encodable for char {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_char(*self);
  }
}
impl Decodable for char {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    decoder.read_char()
  }
}

// str
impl Encodable for str {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_str(self);
  }
}

// String
impl Encodable for String {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_str(self);
  }
}
impl Decodable for String {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    decoder.read_str()
  }
}

// PathBuf
impl Encodable for PathBuf {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_str(&self.to_string_lossy());
  }
}
impl Decodable for PathBuf {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    PathBuf::from(decoder.read_str())
  }
}

// Option
impl<T: Encodable> Encodable for Option<T> {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    match self {
      None => encoder.emit_u8(0),
      Some(val) => {
        encoder.emit_u8(1);
        val.encode(encoder);
      }
    }
  }
}
impl<T: Decodable> Decodable for Option<T> {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    match decoder.read_u8() {
      0 => None,
      _ => Some(T::decode(decoder)),
    }
  }
}

// [T]
impl<T: Encodable> Encodable for [T] {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_u32(self.len() as u32);
    for item in self {
      item.encode(encoder);
    }
  }
}

// &T
impl<T: Encodable + ?Sized> Encodable for &T {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    (**self).encode(encoder);
  }
}

// Vec
impl<T: Encodable> Encodable for Vec<T> {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_u32(self.len() as u32);
    for item in self {
      item.encode(encoder);
    }
  }
}
impl<T: Decodable> Decodable for Vec<T> {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    let len = decoder.read_u32() as usize;
    (0..len).map(|_| T::decode(decoder)).collect()
  }
}

// Box
impl<T: Encodable> Encodable for Box<T> {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    (**self).encode(encoder);
  }
}
impl<T: Decodable> Decodable for Box<T> {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    Box::new(T::decode(decoder))
  }
}

// ()
impl Encodable for () {
  fn encode<E: Encoder + ?Sized>(&self, _encoder: &mut E) {}
}
impl Decodable for () {
  fn decode<D: Decoder + ?Sized>(_decoder: &mut D) -> Self {}
}

// (A,) tuple
impl<A: Encodable> Encodable for (A,) {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    self.0.encode(encoder);
  }
}
impl<A: Decodable> Decodable for (A,) {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    (A::decode(decoder),)
  }
}

// (A, B, C) tuple
impl<A: Encodable, B: Encodable, C: Encodable> Encodable for (A, B, C) {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    self.0.encode(encoder);
    self.1.encode(encoder);
    self.2.encode(encoder);
  }
}
impl<A: Decodable, B: Decodable, C: Decodable> Decodable for (A, B, C) {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    let a = A::decode(decoder);
    let b = B::decode(decoder);
    let c = C::decode(decoder);
    (a, b, c)
  }
}

// (A, B) tuple
impl<A: Encodable, B: Encodable> Encodable for (A, B) {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    self.0.encode(encoder);
    self.1.encode(encoder);
  }
}
impl<A: Decodable, B: Decodable> Decodable for (A, B) {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    let a = A::decode(decoder);
    let b = B::decode(decoder);
    (a, b)
  }
}

// HashMap
impl<K: Encodable + Ord, V: Encodable> Encodable for HashMap<K, V> {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_u32(self.len() as u32);
    let mut entries: Vec<(&K, &V)> = self.iter().collect();
    entries.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
    for (key, value) in entries {
      key.encode(encoder);
      value.encode(encoder);
    }
  }
}
impl<K: Decodable + Eq + Hash, V: Decodable> Decodable for HashMap<K, V> {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    let len = decoder.read_u32() as usize;
    let mut map = HashMap::with_capacity(len);
    for _ in 0..len {
      let key = K::decode(decoder);
      let value = V::decode(decoder);
      map.insert(key, value);
    }
    map
  }
}

use typedown_types::diagnostic::{Diagnostic, DiagnosticCode};
use typedown_types::syntax_kind::SyntaxKind;

// Either
impl<L: Encodable, R: Encodable> Encodable for Either<L, R> {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    match self {
      Either::Left(val) => {
        encoder.emit_u8(0);
        val.encode(encoder);
      }
      Either::Right(val) => {
        encoder.emit_u8(1);
        val.encode(encoder);
      }
    }
  }
}
impl<L: Decodable, R: Decodable> Decodable for Either<L, R> {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    match decoder.read_u8() {
      0 => Either::Left(L::decode(decoder)),
      _ => Either::Right(R::decode(decoder)),
    }
  }
}

// Diagnostic

impl Encodable for Diagnostic {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_u8(self.code() as u8);
    match self {
      Diagnostic::UnexpectedEof {
        expected,
        start_offset,
        end_offset,
      } => {
        expected.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::UnexpectedChar {
        expected,
        encountered,
        start_offset,
        end_offset,
      } => {
        expected.encode(encoder);
        encountered.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::UnterminatedString {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::UnterminatedInterpolation {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::UnterminatedCodeBlock {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::UnterminatedInlineCode {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::UnterminatedMathBlock {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::UnterminatedInlineMath {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::MissingCodeBlockNewline {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::MissingMathBlockNewline {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::InvalidChar {
        encountered,
        start_offset,
        end_offset,
      } => {
        encountered.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::InvalidUtf8 {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::MixedIndentation {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::InconsistentIndentation {
        expected,
        encountered,
        start_offset,
        end_offset,
      } => {
        expected.encode(encoder);
        encountered.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::UnmatchedDedent {
        indent,
        start_offset,
        end_offset,
      } => {
        indent.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::MissingExponentDigits {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::UnexpectedTokensOnFrontmatterMarkerLine {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::MissingFrontmatterMarker { offset } => {
        offset.encode(encoder);
      }
      Diagnostic::MissingMarkdownHeadingHash {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::MissingRequiredSpacesBetweenHashAndHeading {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::MissingSyntaxNode {
        expected,
        start_offset,
        end_offset,
      } => {
        expected.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::UnclosedLink {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::UnclosedBold {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::UnclosedItalic {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::UnclosedStrikethrough {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::UnclosedBoldItalic {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::MismatchedItalicDelimiter {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::MissingExpectMdPrefix {
        expected_prefix,
        start_offset,
        end_offset,
      } => {
        expected_prefix.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::MissingTableSeparatorRow {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::TableColumnCountMismatch {
        expected,
        found,
        start_offset,
        end_offset,
      } => {
        expected.encode(encoder);
        found.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::InsufficientBlockIndent {
        expected_more_than,
        found,
        start_offset,
        end_offset,
      } => {
        expected_more_than.encode(encoder);
        found.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::MissingVaultConfig { root_dir } => {
        root_dir.encode(encoder);
      }
      Diagnostic::VaultConfigReadError { path, message } => {
        path.encode(encoder);
        message.encode(encoder);
      }
      Diagnostic::VaultConfigParseError {
        path,
        message,
        start_offset,
        end_offset,
      } => {
        path.encode(encoder);
        message.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::VaultConfigEmpty { path } => {
        path.encode(encoder);
      }
      Diagnostic::VaultConfigMissingField {
        path,
        field,
        start_offset,
        end_offset,
      } => {
        path.encode(encoder);
        field.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::VaultConfigUnknownField {
        path,
        field,
        start_offset,
        end_offset,
      } => {
        path.encode(encoder);
        field.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::MissingSchemaField {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::UnresolvedSchema {
        name,
        start_offset,
        end_offset,
      } => {
        name.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::WrongTypeArgCount { expected, got } => {
        expected.encode(encoder);
        got.encode(encoder);
      }
      Diagnostic::NotCallable {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::WrongArgCount {
        expected,
        got,
        start_offset,
        end_offset,
      } => {
        expected.encode(encoder);
        got.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::ArgTypeMismatch {
        expected,
        start_offset,
        end_offset,
      } => {
        expected.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::FieldTypeMismatch {
        field,
        expected,
        start_offset,
        end_offset,
      } => {
        field.encode(encoder);
        expected.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::NotIndexable {
        start_offset,
        end_offset,
      } => {
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::IndexTypeMismatch {
        expected,
        start_offset,
        end_offset,
      } => {
        expected.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::TagTypeMismatch {
        expected,
        start_offset,
        end_offset,
      } => {
        expected.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::OperandTypeMismatch {
        op,
        expected,
        start_offset,
        end_offset,
      } => {
        op.encode(encoder);
        expected.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::MissingRequiredField {
        field,
        start_offset,
        end_offset,
      } => {
        field.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::ElementTypeMismatch {
        expected,
        start_offset,
        end_offset,
      } => {
        expected.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::DuplicateKey {
        key,
        start_offset,
        end_offset,
      } => {
        key.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::UnresolvedFileRef {
        path,
        start_offset,
        end_offset,
      } => {
        path.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::UnknownField {
        field,
        on_type,
        start_offset,
        end_offset,
      } => {
        field.encode(encoder);
        on_type.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
      Diagnostic::IndexOutOfBounds {
        index,
        length,
        start_offset,
        end_offset,
      } => {
        index.encode(encoder);
        length.encode(encoder);
        start_offset.encode(encoder);
        end_offset.encode(encoder);
      }
    }
  }
}

impl Decodable for Diagnostic {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    let tag = decoder.read_u8();
    let code =
      DiagnosticCode::try_from(tag).unwrap_or_else(|_| panic!("unknown DiagnosticCode tag {tag}"));
    match code {
      DiagnosticCode::UnexpectedEof => {
        let expected = char::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::UnexpectedEof {
          expected,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::UnexpectedChar => {
        let expected = char::decode(decoder);
        let encountered = char::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::UnexpectedChar {
          expected,
          encountered,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::UnterminatedString => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::UnterminatedString {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::UnterminatedInterpolation => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::UnterminatedInterpolation {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::UnterminatedCodeBlock => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::UnterminatedCodeBlock {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::UnterminatedInlineCode => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::UnterminatedInlineCode {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::UnterminatedMathBlock => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::UnterminatedMathBlock {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::UnterminatedInlineMath => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::UnterminatedInlineMath {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::MissingCodeBlockNewline => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::MissingCodeBlockNewline {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::MissingMathBlockNewline => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::MissingMathBlockNewline {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::InvalidChar => {
        let encountered = char::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::InvalidChar {
          encountered,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::InvalidUtf8 => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::InvalidUtf8 {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::MixedIndentation => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::MixedIndentation {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::InconsistentIndentation => {
        let expected = char::decode(decoder);
        let encountered = char::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::InconsistentIndentation {
          expected,
          encountered,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::UnmatchedDedent => {
        let indent = usize::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::UnmatchedDedent {
          indent,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::MissingExponentDigits => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::MissingExponentDigits {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::UnexpectedTokensOnFrontmatterMarkerLine => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::UnexpectedTokensOnFrontmatterMarkerLine {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::MissingFrontmatterMarker => {
        let offset = usize::decode(decoder);
        Diagnostic::MissingFrontmatterMarker { offset }
      }
      DiagnosticCode::MissingMarkdownHeadingHash => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::MissingMarkdownHeadingHash {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::MissingRequiredSpacesBetweenHashAndHeading => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::MissingRequiredSpacesBetweenHashAndHeading {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::MissingSyntaxNode => {
        let expected = SyntaxKind::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::MissingSyntaxNode {
          expected,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::UnclosedLink => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::UnclosedLink {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::UnclosedBold => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::UnclosedBold {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::UnclosedItalic => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::UnclosedItalic {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::UnclosedStrikethrough => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::UnclosedStrikethrough {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::UnclosedBoldItalic => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::UnclosedBoldItalic {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::MismatchedItalicDelimiter => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::MismatchedItalicDelimiter {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::MissingExpectMdPrefix => {
        let expected_prefix = String::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::MissingExpectMdPrefix {
          expected_prefix,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::MissingTableSeparatorRow => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::MissingTableSeparatorRow {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::TableColumnCountMismatch => {
        let expected = usize::decode(decoder);
        let found = usize::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::TableColumnCountMismatch {
          expected,
          found,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::InsufficientBlockIndent => {
        let expected_more_than = usize::decode(decoder);
        let found = usize::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::InsufficientBlockIndent {
          expected_more_than,
          found,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::MissingVaultConfig => {
        let root_dir = String::decode(decoder);
        Diagnostic::MissingVaultConfig { root_dir }
      }
      DiagnosticCode::VaultConfigReadError => {
        let path = String::decode(decoder);
        let message = String::decode(decoder);
        Diagnostic::VaultConfigReadError { path, message }
      }
      DiagnosticCode::VaultConfigParseError => {
        let path = String::decode(decoder);
        let message = String::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::VaultConfigParseError {
          path,
          message,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::VaultConfigEmpty => {
        let path = String::decode(decoder);
        Diagnostic::VaultConfigEmpty { path }
      }
      DiagnosticCode::VaultConfigMissingField => {
        let path = String::decode(decoder);
        let field = String::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::VaultConfigMissingField {
          path,
          field,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::VaultConfigUnknownField => {
        let path = String::decode(decoder);
        let field = String::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::VaultConfigUnknownField {
          path,
          field,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::MissingSchemaField => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::MissingSchemaField {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::UnresolvedSchema => {
        let name = String::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::UnresolvedSchema {
          name,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::WrongTypeArgCount => {
        let expected = usize::decode(decoder);
        let got = usize::decode(decoder);
        Diagnostic::WrongTypeArgCount { expected, got }
      }
      DiagnosticCode::NotCallable => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::NotCallable {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::WrongArgCount => {
        let expected = usize::decode(decoder);
        let got = usize::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::WrongArgCount {
          expected,
          got,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::ArgTypeMismatch => {
        let expected = String::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::ArgTypeMismatch {
          expected,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::FieldTypeMismatch => {
        let field = String::decode(decoder);
        let expected = String::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::FieldTypeMismatch {
          field,
          expected,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::NotIndexable => {
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::NotIndexable {
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::IndexTypeMismatch => {
        let expected = String::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::IndexTypeMismatch {
          expected,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::TagTypeMismatch => {
        let expected = String::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::TagTypeMismatch {
          expected,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::OperandTypeMismatch => {
        let op = String::decode(decoder);
        let expected = String::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::OperandTypeMismatch {
          op,
          expected,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::MissingRequiredField => {
        let field = String::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::MissingRequiredField {
          field,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::ElementTypeMismatch => {
        let expected = String::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::ElementTypeMismatch {
          expected,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::DuplicateKey => {
        let key = String::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::DuplicateKey {
          key,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::UnresolvedFileRef => {
        let path = String::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::UnresolvedFileRef {
          path,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::UnknownField => {
        let field = String::decode(decoder);
        let on_type = String::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::UnknownField {
          field,
          on_type,
          start_offset,
          end_offset,
        }
      }
      DiagnosticCode::IndexOutOfBounds => {
        let index = usize::decode(decoder);
        let length = usize::decode(decoder);
        let start_offset = usize::decode(decoder);
        let end_offset = usize::decode(decoder);
        Diagnostic::IndexOutOfBounds {
          index,
          length,
          start_offset,
          end_offset,
        }
      }
    }
  }
}

// SyntaxKind

impl Encodable for SyntaxKind {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_u16(*self as u16);
  }
}

impl Decodable for SyntaxKind {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    let val = decoder.read_u16();
    unsafe { std::mem::transmute::<u16, SyntaxKind>(val) }
  }
}
