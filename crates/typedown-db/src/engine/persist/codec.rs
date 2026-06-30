use std::collections::HashMap;
use std::hash::Hash;
use std::path::PathBuf;

use typedown_types::either::Either;

use crate::QueryDatabase;

// Object-safe encoder trait
pub trait Encoder {
  fn db(&self) -> &dyn QueryDatabase;
  fn emit_u8(&mut self, v: u8);
  fn emit_u16(&mut self, v: u16);
  fn emit_u32(&mut self, v: u32);
  fn emit_u64(&mut self, v: u64);
  fn emit_usize(&mut self, v: usize);
  fn emit_i8(&mut self, v: i8);
  fn emit_i32(&mut self, v: i32);
  fn emit_i64(&mut self, v: i64);
  fn emit_f64(&mut self, v: f64);
  fn emit_bool(&mut self, v: bool);
  fn emit_char(&mut self, v: char);
  fn emit_str(&mut self, v: &str);
  fn emit_bytes(&mut self, v: &[u8]);
  fn intern_blob(&mut self, blob: Vec<u8>, hint: Option<usize>) -> u32;
}

// Object-safe decoder trait
pub trait Decoder {
  fn db(&self) -> &dyn QueryDatabase;
  fn read_u8(&mut self) -> u8;
  fn read_u16(&mut self) -> u16;
  fn read_u32(&mut self) -> u32;
  fn read_u64(&mut self) -> u64;
  fn read_usize(&mut self) -> usize;
  fn read_i8(&mut self) -> i8;
  fn read_i32(&mut self) -> i32;
  fn read_i64(&mut self) -> i64;
  fn read_f64(&mut self) -> f64;
  fn read_bool(&mut self) -> bool;
  fn read_char(&mut self) -> char;
  fn read_str(&mut self) -> String;
  fn read_bytes_owned(&mut self) -> Vec<u8>;
  fn get_intern_blob(&self, index: u32) -> &[u8];
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
