use std::collections::HashMap;
use std::hash::Hash;
use std::path::PathBuf;

use typedown_types::either::Either;

use crate::QueryDatabase;

pub struct Encoder<'a> {
  db: &'a dyn QueryDatabase,
  intern_hints: HashMap<usize, u32>,
  intern_blobs: Vec<Vec<u8>>,
  intern_table: HashMap<Vec<u8>, u32>,
}

impl<'a> Encoder<'a> {
  pub fn new(db: &'a dyn QueryDatabase) -> Self {
    Self {
      db,
      intern_hints: HashMap::new(),
      intern_blobs: Vec::new(),
      intern_table: HashMap::new(),
    }
  }

  pub fn db(&self) -> &dyn QueryDatabase {
    self.db
  }

  pub fn finish(self) -> Vec<Vec<u8>> {
    self.intern_blobs
  }

  pub fn intern_blob(&mut self, blob: Vec<u8>, hint: Option<usize>) -> u32 {
    if let Some(key) = hint {
      if let Some(&index) = self.intern_hints.get(&key) {
        return index;
      }
    }
    if let Some(&index) = self.intern_table.get(&blob) {
      if let Some(key) = hint {
        self.intern_hints.insert(key, index);
      }
      return index;
    }
    let index = self.intern_blobs.len() as u32;
    self.intern_table.insert(blob.clone(), index);
    self.intern_blobs.push(blob);
    if let Some(key) = hint {
      self.intern_hints.insert(key, index);
    }
    index
  }

  pub fn emit_raw(buf: &mut Vec<u8>, bytes: &[u8]) -> usize {
    buf.extend_from_slice(bytes);
    bytes.len()
  }

  pub fn emit_u8(buf: &mut Vec<u8>, v: u8) -> usize {
    Self::emit_raw(buf, &[v])
  }
  pub fn emit_u16(buf: &mut Vec<u8>, v: u16) -> usize {
    Self::emit_raw(buf, &v.to_le_bytes())
  }
  pub fn emit_u32(buf: &mut Vec<u8>, v: u32) -> usize {
    Self::emit_raw(buf, &v.to_le_bytes())
  }
  pub fn emit_u64(buf: &mut Vec<u8>, v: u64) -> usize {
    Self::emit_raw(buf, &v.to_le_bytes())
  }
  pub fn emit_u128(buf: &mut Vec<u8>, v: u128) -> usize {
    Self::emit_raw(buf, &v.to_le_bytes())
  }
  pub fn emit_usize(buf: &mut Vec<u8>, v: usize) -> usize {
    Self::emit_u64(buf, v as u64)
  }
  pub fn emit_isize(buf: &mut Vec<u8>, v: isize) -> usize {
    Self::emit_i64(buf, v as i64)
  }
  pub fn emit_i8(buf: &mut Vec<u8>, v: i8) -> usize {
    Self::emit_raw(buf, &[v as u8])
  }
  pub fn emit_i16(buf: &mut Vec<u8>, v: i16) -> usize {
    Self::emit_raw(buf, &v.to_le_bytes())
  }
  pub fn emit_i32(buf: &mut Vec<u8>, v: i32) -> usize {
    Self::emit_raw(buf, &v.to_le_bytes())
  }
  pub fn emit_i64(buf: &mut Vec<u8>, v: i64) -> usize {
    Self::emit_raw(buf, &v.to_le_bytes())
  }
  pub fn emit_i128(buf: &mut Vec<u8>, v: i128) -> usize {
    Self::emit_raw(buf, &v.to_le_bytes())
  }
  pub fn emit_f64(buf: &mut Vec<u8>, v: f64) -> usize {
    Self::emit_raw(buf, &v.to_le_bytes())
  }
  pub fn emit_bool(buf: &mut Vec<u8>, v: bool) -> usize {
    Self::emit_u8(buf, v as u8)
  }
  pub fn emit_char(buf: &mut Vec<u8>, v: char) -> usize {
    Self::emit_u32(buf, v as u32)
  }
  pub fn emit_str(buf: &mut Vec<u8>, v: &str) -> usize {
    Self::emit_u32(buf, v.len() as u32) + Self::emit_raw(buf, v.as_bytes())
  }
  pub fn emit_bytes(buf: &mut Vec<u8>, v: &[u8]) -> usize {
    Self::emit_u32(buf, v.len() as u32) + Self::emit_raw(buf, v)
  }
}

pub struct Decoder<'a> {
  db: &'a dyn QueryDatabase,
  intern_blobs: Vec<Vec<u8>>,
}

impl<'a> Decoder<'a> {
  pub fn new(db: &'a dyn QueryDatabase, intern_blobs: Vec<Vec<u8>>) -> Self {
    Self { db, intern_blobs }
  }

  pub fn db(&self) -> &dyn QueryDatabase {
    self.db
  }

  pub fn get_intern_blob(&self, index: u32) -> &[u8] {
    &self.intern_blobs[index as usize]
  }

  pub fn read_raw(data: &mut &[u8], buf: &mut [u8]) -> usize {
    let len = buf.len();
    buf.copy_from_slice(&data[..len]);
    *data = &data[len..];
    len
  }

  pub fn read_u8(data: &mut &[u8]) -> u8 {
    let mut buf = [0u8; 1];
    Self::read_raw(data, &mut buf);
    buf[0]
  }
  pub fn read_u16(data: &mut &[u8]) -> u16 {
    let mut buf = [0u8; 2];
    Self::read_raw(data, &mut buf);
    u16::from_le_bytes(buf)
  }
  pub fn read_u32(data: &mut &[u8]) -> u32 {
    let mut buf = [0u8; 4];
    Self::read_raw(data, &mut buf);
    u32::from_le_bytes(buf)
  }
  pub fn read_u64(data: &mut &[u8]) -> u64 {
    let mut buf = [0u8; 8];
    Self::read_raw(data, &mut buf);
    u64::from_le_bytes(buf)
  }
  pub fn read_u128(data: &mut &[u8]) -> u128 {
    let mut buf = [0u8; 16];
    Self::read_raw(data, &mut buf);
    u128::from_le_bytes(buf)
  }
  pub fn read_usize(data: &mut &[u8]) -> usize {
    Self::read_u64(data) as usize
  }
  pub fn read_isize(data: &mut &[u8]) -> isize {
    Self::read_i64(data) as isize
  }
  pub fn read_i8(data: &mut &[u8]) -> i8 {
    Self::read_u8(data) as i8
  }
  pub fn read_i16(data: &mut &[u8]) -> i16 {
    let mut buf = [0u8; 2];
    Self::read_raw(data, &mut buf);
    i16::from_le_bytes(buf)
  }
  pub fn read_i32(data: &mut &[u8]) -> i32 {
    let mut buf = [0u8; 4];
    Self::read_raw(data, &mut buf);
    i32::from_le_bytes(buf)
  }
  pub fn read_i64(data: &mut &[u8]) -> i64 {
    let mut buf = [0u8; 8];
    Self::read_raw(data, &mut buf);
    i64::from_le_bytes(buf)
  }
  pub fn read_i128(data: &mut &[u8]) -> i128 {
    let mut buf = [0u8; 16];
    Self::read_raw(data, &mut buf);
    i128::from_le_bytes(buf)
  }
  pub fn read_f64(data: &mut &[u8]) -> f64 {
    let mut buf = [0u8; 8];
    Self::read_raw(data, &mut buf);
    f64::from_le_bytes(buf)
  }
  pub fn read_bool(data: &mut &[u8]) -> bool {
    Self::read_u8(data) != 0
  }
  pub fn read_char(data: &mut &[u8]) -> char {
    char::from_u32(Self::read_u32(data)).unwrap()
  }
  pub fn read_str(data: &mut &[u8]) -> String {
    let len = Self::read_u32(data) as usize;
    let mut buf = vec![0u8; len];
    Self::read_raw(data, &mut buf);
    String::from_utf8(buf).unwrap()
  }
  pub fn read_bytes_owned(data: &mut &[u8]) -> Vec<u8> {
    let len = Self::read_u32(data) as usize;
    let mut buf = vec![0u8; len];
    Self::read_raw(data, &mut buf);
    buf
  }
}

// Encodable / Decodable traits
pub trait Encodable {
  fn encode(&self, buf: &mut Vec<u8>, encoder: &mut Encoder);
}

pub trait Decodable: Sized {
  fn decode(data: &mut &[u8], decoder: &Decoder) -> Self;
}

/* Primitive implementations */

// u8
impl Encodable for u8 {
  fn encode(&self, buf: &mut Vec<u8>, _encoder: &mut Encoder) {
    Encoder::emit_u8(buf, *self);
  }
}
impl Decodable for u8 {
  fn decode(data: &mut &[u8], _decoder: &Decoder) -> Self {
    Decoder::read_u8(data)
  }
}

// u16
impl Encodable for u16 {
  fn encode(&self, buf: &mut Vec<u8>, _encoder: &mut Encoder) {
    Encoder::emit_u16(buf, *self);
  }
}
impl Decodable for u16 {
  fn decode(data: &mut &[u8], _decoder: &Decoder) -> Self {
    Decoder::read_u16(data)
  }
}

// u32
impl Encodable for u32 {
  fn encode(&self, buf: &mut Vec<u8>, _encoder: &mut Encoder) {
    Encoder::emit_u32(buf, *self);
  }
}
impl Decodable for u32 {
  fn decode(data: &mut &[u8], _decoder: &Decoder) -> Self {
    Decoder::read_u32(data)
  }
}

// u64
impl Encodable for u64 {
  fn encode(&self, buf: &mut Vec<u8>, _encoder: &mut Encoder) {
    Encoder::emit_u64(buf, *self);
  }
}
impl Decodable for u64 {
  fn decode(data: &mut &[u8], _decoder: &Decoder) -> Self {
    Decoder::read_u64(data)
  }
}

// usize (encoded as u64)
impl Encodable for usize {
  fn encode(&self, buf: &mut Vec<u8>, _encoder: &mut Encoder) {
    Encoder::emit_usize(buf, *self);
  }
}
impl Decodable for usize {
  fn decode(data: &mut &[u8], _decoder: &Decoder) -> Self {
    Decoder::read_usize(data)
  }
}

// u128
impl Encodable for u128 {
  fn encode(&self, buf: &mut Vec<u8>, _encoder: &mut Encoder) {
    Encoder::emit_u128(buf, *self);
  }
}
impl Decodable for u128 {
  fn decode(data: &mut &[u8], _decoder: &Decoder) -> Self {
    Decoder::read_u128(data)
  }
}

// isize
impl Encodable for isize {
  fn encode(&self, buf: &mut Vec<u8>, _encoder: &mut Encoder) {
    Encoder::emit_isize(buf, *self);
  }
}
impl Decodable for isize {
  fn decode(data: &mut &[u8], _decoder: &Decoder) -> Self {
    Decoder::read_isize(data)
  }
}

// i8
impl Encodable for i8 {
  fn encode(&self, buf: &mut Vec<u8>, _encoder: &mut Encoder) {
    Encoder::emit_i8(buf, *self);
  }
}
impl Decodable for i8 {
  fn decode(data: &mut &[u8], _decoder: &Decoder) -> Self {
    Decoder::read_i8(data)
  }
}

// i16
impl Encodable for i16 {
  fn encode(&self, buf: &mut Vec<u8>, _encoder: &mut Encoder) {
    Encoder::emit_i16(buf, *self);
  }
}
impl Decodable for i16 {
  fn decode(data: &mut &[u8], _decoder: &Decoder) -> Self {
    Decoder::read_i16(data)
  }
}

// i32
impl Encodable for i32 {
  fn encode(&self, buf: &mut Vec<u8>, _encoder: &mut Encoder) {
    Encoder::emit_i32(buf, *self);
  }
}
impl Decodable for i32 {
  fn decode(data: &mut &[u8], _decoder: &Decoder) -> Self {
    Decoder::read_i32(data)
  }
}

// i64
impl Encodable for i64 {
  fn encode(&self, buf: &mut Vec<u8>, _encoder: &mut Encoder) {
    Encoder::emit_i64(buf, *self);
  }
}
impl Decodable for i64 {
  fn decode(data: &mut &[u8], _decoder: &Decoder) -> Self {
    Decoder::read_i64(data)
  }
}

// i128
impl Encodable for i128 {
  fn encode(&self, buf: &mut Vec<u8>, _encoder: &mut Encoder) {
    Encoder::emit_i128(buf, *self);
  }
}
impl Decodable for i128 {
  fn decode(data: &mut &[u8], _decoder: &Decoder) -> Self {
    Decoder::read_i128(data)
  }
}

// f64
impl Encodable for f64 {
  fn encode(&self, buf: &mut Vec<u8>, _encoder: &mut Encoder) {
    Encoder::emit_f64(buf, *self);
  }
}
impl Decodable for f64 {
  fn decode(data: &mut &[u8], _decoder: &Decoder) -> Self {
    Decoder::read_f64(data)
  }
}

// bool
impl Encodable for bool {
  fn encode(&self, buf: &mut Vec<u8>, _encoder: &mut Encoder) {
    Encoder::emit_bool(buf, *self);
  }
}
impl Decodable for bool {
  fn decode(data: &mut &[u8], _decoder: &Decoder) -> Self {
    Decoder::read_bool(data)
  }
}

// char
impl Encodable for char {
  fn encode(&self, buf: &mut Vec<u8>, _encoder: &mut Encoder) {
    Encoder::emit_char(buf, *self);
  }
}
impl Decodable for char {
  fn decode(data: &mut &[u8], _decoder: &Decoder) -> Self {
    Decoder::read_char(data)
  }
}

// str
impl Encodable for str {
  fn encode(&self, buf: &mut Vec<u8>, _encoder: &mut Encoder) {
    Encoder::emit_str(buf, self);
  }
}

// String
impl Encodable for String {
  fn encode(&self, buf: &mut Vec<u8>, _encoder: &mut Encoder) {
    Encoder::emit_str(buf, self);
  }
}
impl Decodable for String {
  fn decode(data: &mut &[u8], _decoder: &Decoder) -> Self {
    Decoder::read_str(data)
  }
}

// PathBuf
impl Encodable for PathBuf {
  fn encode(&self, buf: &mut Vec<u8>, _encoder: &mut Encoder) {
    Encoder::emit_str(buf, &self.to_string_lossy());
  }
}
impl Decodable for PathBuf {
  fn decode(data: &mut &[u8], _decoder: &Decoder) -> Self {
    PathBuf::from(Decoder::read_str(data))
  }
}

// Option
impl<T: Encodable> Encodable for Option<T> {
  fn encode(&self, buf: &mut Vec<u8>, encoder: &mut Encoder) {
    match self {
      None => {
        Encoder::emit_u8(buf, 0);
      }
      Some(val) => {
        Encoder::emit_u8(buf, 1);
        val.encode(buf, encoder);
      }
    }
  }
}
impl<T: Decodable> Decodable for Option<T> {
  fn decode(data: &mut &[u8], decoder: &Decoder) -> Self {
    match Decoder::read_u8(data) {
      0 => None,
      _ => Some(T::decode(data, decoder)),
    }
  }
}

// [T]
impl<T: Encodable> Encodable for [T] {
  fn encode(&self, buf: &mut Vec<u8>, encoder: &mut Encoder) {
    Encoder::emit_u32(buf, self.len() as u32);
    for item in self {
      item.encode(buf, encoder);
    }
  }
}

// &T
impl<T: Encodable + ?Sized> Encodable for &T {
  fn encode(&self, buf: &mut Vec<u8>, encoder: &mut Encoder) {
    (**self).encode(buf, encoder);
  }
}

// Vec
impl<T: Encodable> Encodable for Vec<T> {
  fn encode(&self, buf: &mut Vec<u8>, encoder: &mut Encoder) {
    Encoder::emit_u32(buf, self.len() as u32);
    for item in self {
      item.encode(buf, encoder);
    }
  }
}
impl<T: Decodable> Decodable for Vec<T> {
  fn decode(data: &mut &[u8], decoder: &Decoder) -> Self {
    let len = Decoder::read_u32(data) as usize;
    (0..len).map(|_| T::decode(data, decoder)).collect()
  }
}

// Box
impl<T: Encodable> Encodable for Box<T> {
  fn encode(&self, buf: &mut Vec<u8>, encoder: &mut Encoder) {
    (**self).encode(buf, encoder);
  }
}
impl<T: Decodable> Decodable for Box<T> {
  fn decode(data: &mut &[u8], decoder: &Decoder) -> Self {
    Box::new(T::decode(data, decoder))
  }
}

// ()
impl Encodable for () {
  fn encode(&self, _buf: &mut Vec<u8>, _encoder: &mut Encoder) {}
}
impl Decodable for () {
  fn decode(_data: &mut &[u8], _decoder: &Decoder) -> Self {}
}

// (A,) tuple
impl<A: Encodable> Encodable for (A,) {
  fn encode(&self, buf: &mut Vec<u8>, encoder: &mut Encoder) {
    self.0.encode(buf, encoder);
  }
}
impl<A: Decodable> Decodable for (A,) {
  fn decode(data: &mut &[u8], decoder: &Decoder) -> Self {
    (A::decode(data, decoder),)
  }
}

// (A, B, C) tuple
impl<A: Encodable, B: Encodable, C: Encodable> Encodable for (A, B, C) {
  fn encode(&self, buf: &mut Vec<u8>, encoder: &mut Encoder) {
    self.0.encode(buf, encoder);
    self.1.encode(buf, encoder);
    self.2.encode(buf, encoder);
  }
}
impl<A: Decodable, B: Decodable, C: Decodable> Decodable for (A, B, C) {
  fn decode(data: &mut &[u8], decoder: &Decoder) -> Self {
    let a = A::decode(data, decoder);
    let b = B::decode(data, decoder);
    let c = C::decode(data, decoder);
    (a, b, c)
  }
}

// (A, B) tuple
impl<A: Encodable, B: Encodable> Encodable for (A, B) {
  fn encode(&self, buf: &mut Vec<u8>, encoder: &mut Encoder) {
    self.0.encode(buf, encoder);
    self.1.encode(buf, encoder);
  }
}
impl<A: Decodable, B: Decodable> Decodable for (A, B) {
  fn decode(data: &mut &[u8], decoder: &Decoder) -> Self {
    let a = A::decode(data, decoder);
    let b = B::decode(data, decoder);
    (a, b)
  }
}

// HashMap
impl<K: Encodable + Ord, V: Encodable> Encodable for HashMap<K, V> {
  fn encode(&self, buf: &mut Vec<u8>, encoder: &mut Encoder) {
    Encoder::emit_u32(buf, self.len() as u32);
    let mut entries: Vec<(&K, &V)> = self.iter().collect();
    entries.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
    for (key, value) in entries {
      key.encode(buf, encoder);
      value.encode(buf, encoder);
    }
  }
}
impl<K: Decodable + Eq + Hash, V: Decodable> Decodable for HashMap<K, V> {
  fn decode(data: &mut &[u8], decoder: &Decoder) -> Self {
    let len = Decoder::read_u32(data) as usize;
    let mut map = HashMap::with_capacity(len);
    for _ in 0..len {
      let key = K::decode(data, decoder);
      let value = V::decode(data, decoder);
      map.insert(key, value);
    }
    map
  }
}

// Either
impl<L: Encodable, R: Encodable> Encodable for Either<L, R> {
  fn encode(&self, buf: &mut Vec<u8>, encoder: &mut Encoder) {
    match self {
      Either::Left(val) => {
        Encoder::emit_u8(buf, 0);
        val.encode(buf, encoder);
      }
      Either::Right(val) => {
        Encoder::emit_u8(buf, 1);
        val.encode(buf, encoder);
      }
    }
  }
}
impl<L: Decodable, R: Decodable> Decodable for Either<L, R> {
  fn decode(data: &mut &[u8], decoder: &Decoder) -> Self {
    match Decoder::read_u8(data) {
      0 => Either::Left(L::decode(data, decoder)),
      _ => Either::Right(R::decode(data, decoder)),
    }
  }
}
