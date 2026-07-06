use std::collections::HashMap;

use crate::syntax::green::cache::with_green_cache;
use crate::syntax::green::node::SyntaxNode;
use crate::syntax::green::token::SyntaxToken;
use crate::syntax::red::RedNode;
use crate::{db::types::FileHandle, syntax::green::GreenNode};
use typedown_types::syntax_kind::SyntaxKind;

use crate::db::TypedownDatabase;
use typedown_incremental::{
  Decodable, Decoder, Encodable, Encoder, QueryDatabase, StableHash, StableHasher,
};

// TypedownEncoder
pub struct TypedownEncoder<'a> {
  pub db: &'a TypedownDatabase,
  pub buf: Vec<u8>,

  /// Map from an interned hint to an index in the interned blob table
  /// An interned hint is like a tag for the interned value
  pub intern_hints: HashMap<usize, u32>,
  /// A simple list of blobs
  /// The index in this list is used in interned hint
  pub intern_blobs: Vec<Vec<u8>>,
  /// Reverse map from a blob to its index
  pub intern_table: HashMap<Vec<u8>, u32>,
}

impl<'a> TypedownEncoder<'a> {
  pub fn new(db: &'a TypedownDatabase) -> Self {
    TypedownEncoder {
      db,
      buf: Vec::new(),
      intern_hints: HashMap::new(),
      intern_table: HashMap::new(),
      intern_blobs: Vec::new(),
    }
  }

  pub fn finish(self) -> Vec<u8> {
    self.buf
  }
}

impl Encoder for TypedownEncoder<'_> {
  fn db(&self) -> &dyn QueryDatabase {
    self.db
  }

  fn emit_raw(&mut self, bytes: &[u8]) {
    self.buf.extend_from_slice(bytes);
  }

  fn intern_blob(&mut self, blob: Vec<u8>, hint: Option<usize>) -> u32 {
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
}

// TypedownDecoder
pub struct TypedownDecoder<'a> {
  pub db: &'a TypedownDatabase,
  data: &'a [u8],
  pos: usize,
  // Interned blobs loaded from disk.
  pub intern_blobs: &'a [Vec<u8>],
}

impl<'a> TypedownDecoder<'a> {
  pub fn new(db: &'a TypedownDatabase, data: &'a [u8]) -> Self {
    TypedownDecoder {
      db,
      data,
      pos: 0,
      intern_blobs: &[],
    }
  }

  pub fn with_intern_blobs(
    db: &'a TypedownDatabase,
    data: &'a [u8],
    intern_blobs: &'a [Vec<u8>],
  ) -> Self {
    TypedownDecoder {
      db,
      data,
      pos: 0,
      intern_blobs,
    }
  }

  pub fn position(&self) -> usize {
    self.pos
  }
}

impl Decoder for TypedownDecoder<'_> {
  fn db(&self) -> &dyn QueryDatabase {
    self.db
  }

  fn read_raw(&mut self, buf: &mut [u8]) {
    buf.copy_from_slice(&self.data[self.pos..self.pos + buf.len()]);
    self.pos += buf.len();
  }

  fn get_intern_blob(&self, index: u32) -> &[u8] {
    &self.intern_blobs[index as usize]
  }
}

// GreenNode (interned)
fn encode_green_node<E: Encoder + ?Sized>(node: &GreenNode, encoder: &mut E) -> u32 {
  let hint = Some(node.as_ptr());
  if node.is_node() {
    let syntax_node = node.as_node().unwrap();
    let children = syntax_node.children();
    let mut child_indices = Vec::with_capacity(children.len());
    for child in children {
      child_indices.push(encode_green_node(&child, encoder));
    }
    let mut blob = Vec::new();
    blob.push(0); // tag: node
    blob.extend_from_slice(&(syntax_node.kind() as u16).to_le_bytes());
    blob.extend_from_slice(&(child_indices.len() as u32).to_le_bytes());
    for idx in &child_indices {
      blob.extend_from_slice(&idx.to_le_bytes());
    }
    encoder.intern_blob(blob, hint)
  } else {
    let token = node.as_token().unwrap();
    let mut blob = Vec::new();
    blob.push(1); // tag: token
    blob.extend_from_slice(&(token.kind() as u16).to_le_bytes());
    let bytes = token.bytes();
    blob.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    blob.extend_from_slice(bytes);
    encoder.intern_blob(blob, hint)
  }
}

impl Encodable for GreenNode {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    let index = encode_green_node(self, encoder);
    encoder.emit_u32(index);
  }
}

fn decode_green_blob<D: Decoder + ?Sized>(index: usize, decoder: &D) -> GreenNode {
  let blob = decoder.get_intern_blob(index as u32);
  let tag = blob[0];
  let kind_val = u16::from_le_bytes(blob[1..3].try_into().unwrap());

  let kind = unsafe { std::mem::transmute::<u16, SyntaxKind>(kind_val) };

  match tag {
    0 => {
      let child_count = u32::from_le_bytes(blob[3..7].try_into().unwrap()) as usize;
      let mut children = Vec::with_capacity(child_count);
      for idx in 0..child_count {
        let offset = 7 + idx * 4;
        let child_index = u32::from_le_bytes(blob[offset..offset + 4].try_into().unwrap()) as usize;
        let child = decode_green_blob(child_index, decoder);
        children.push(child);
      }
      let syntax_node = with_green_cache(|cache| cache.node(kind, &children));
      GreenNode::from_node(syntax_node)
    }
    1 => {
      let byte_len = u32::from_le_bytes(blob[3..7].try_into().unwrap()) as usize;
      let bytes = &blob[7..7 + byte_len];
      let token = with_green_cache(|cache| cache.token(kind, bytes));
      GreenNode::from_token(token)
    }
    _ => panic!("unknown GreenNode tag {tag}"),
  }
}

impl Decodable for GreenNode {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    let index = decoder.read_u32() as usize;
    decode_green_blob(index, decoder)
  }
}

// RedNode
impl Encodable for RedNode {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    self.offset().encode(encoder);
    let root = self.root();
    (*root).encode(encoder);
  }
}

impl Decodable for RedNode {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    let offset = usize::decode(decoder);
    let green = GreenNode::decode(decoder);
    let root_node = green.as_node().expect("RedNode root must be a node");
    let root = RedNode::new_root(root_node.clone());
    root.find_at_offset(offset).unwrap_or(root)
  }
}

// StableHash impls for syntax types

impl StableHash for SyntaxToken {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    self.kind().stable_hash(db, hasher);
    self.bytes().stable_hash(db, hasher);
  }
}

impl StableHash for SyntaxNode {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    self.kind().stable_hash(db, hasher);
    self.children().stable_hash(db, hasher);
  }
}

impl StableHash for GreenNode {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    if self.is_node() {
      std::hash::Hasher::write_u8(hasher, 0);
      self.as_node().unwrap().stable_hash(db, hasher);
    } else {
      std::hash::Hasher::write_u8(hasher, 1);
      self.as_token().unwrap().stable_hash(db, hasher);
    }
  }
}

impl StableHash for RedNode {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    (self.offset() as u64).stable_hash(db, hasher);
    (**self).stable_hash(db, hasher);
  }
}

impl StableHash for FileHandle {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, db: &DB, hasher: &mut StableHasher) {
    std::mem::discriminant(self).stable_hash(db, hasher);
    match self {
      FileHandle::Path(path, content) => {
        path.stable_hash(db, hasher);
        content.stable_hash(db, hasher);
      }
      FileHandle::Content(content) => {
        content.stable_hash(db, hasher);
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fmt::Debug;

  use proptest::prelude::*;

  use typedown_incremental::{Decodable, Encodable};
  use typedown_incremental::{Decoder, Encoder, QueryStorage};

  use crate::db::{TypedownDatabase, TypedownDecoder, TypedownEncoder};

  /// Check that encode and decode return the original value
  fn encode_decode_roundtrip<T: Encodable + Decodable + PartialEq + Debug>(v: &T) {
    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };
    let mut enc = TypedownEncoder::new(&db);
    v.encode(&mut enc);
    let bytes = enc.finish();
    let mut dec = TypedownDecoder::new(&db, &bytes);
    let decoded = T::decode(&mut dec);
    assert_eq!(*v, decoded);
  }

  // Boolean encode/decode
  #[test]
  fn encode_bool_false_correctly() {
    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };

    let mut encoder = TypedownEncoder::new(&db);
    encoder.emit_bool(false);
    let bytes = encoder.finish();

    assert_eq!(bytes, vec![0]);
  }

  #[test]
  fn encode_bool_true_correctly() {
    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };

    let mut encoder = TypedownEncoder::new(&db);
    encoder.emit_bool(true);
    let bytes = encoder.finish();

    assert_eq!(bytes, vec![1]);
  }

  #[test]
  fn decode_bool_false_correctly() {
    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };

    let data = vec![0];
    let mut decoder = TypedownDecoder::new(&db, &data);

    let decoded_value = decoder.read_bool();
    assert_eq!(decoded_value, false);
  }

  #[test]
  fn decode_bool_true_correctly() {
    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };

    let data = vec![1];
    let mut decoder = TypedownDecoder::new(&db, &data);

    let decoded_value = decoder.read_bool();
    assert_eq!(decoded_value, true);
  }

  proptest! {
    #[test]
    fn bool_roundtrip(v in any::<bool>()) {
      encode_decode_roundtrip(&v);
    }
  }

  // Number encode/decode

  proptest! {
    #[test]
    fn char_rountrip(v in any::<char>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn u8_roundtrip(v in any::<u8>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn i8_roundtrip(v in any::<i8>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn i16_roundtrip(v in any::<i16>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn u16_roundtrip(v in any::<u16>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn i32_roundtrip(v in any::<i32>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn u32_roundtrip(v in any::<u32>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn i64_roundtrip(v in any::<i64>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn u64_roundtrip(v in any::<u64>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn i128_roundtrip(v in any::<i128>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn u128_roundtrip(v in any::<u128>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn usize_roundtrip(v in any::<usize>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn isize_roundtrip(v in any::<isize>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn f64_roundtrip(v in any::<f64>()) {
      encode_decode_roundtrip(&v);
    }
  }
}
