use std::collections::HashMap;

use crate::syntax::diagnostic::{Diagnostic, DiagnosticCode};
use crate::syntax::green::cache::with_green_cache;
use crate::syntax::green::node::SyntaxNode;
use crate::syntax::green::token::SyntaxToken;
use crate::syntax::red::RedNode;
use crate::syntax::syntax_kind::SyntaxKind;
use crate::{db::types::FileHandle, syntax::green::GreenNode};

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

  pub fn finish(self) -> (Vec<u8>, Vec<Vec<u8>>) {
    (self.buf, self.intern_blobs)
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
  pub fn new(db: &'a TypedownDatabase, data: &'a [u8], intern_blobs: &'a [Vec<u8>]) -> Self {
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

  let kind =
    SyntaxKind::from_repr(kind_val).unwrap_or_else(|| panic!("unknown SyntaxKind {kind_val}"));

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

// SyntaxKind

impl Encodable for SyntaxKind {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    encoder.emit_u16(*self as u16);
  }
}

impl Decodable for SyntaxKind {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    let val = decoder.read_u16();
    SyntaxKind::from_repr(val).unwrap_or_else(|| panic!("unknown SyntaxKind {val}"))
  }
}

impl StableHash for SyntaxKind {
  fn stable_hash<DB: QueryDatabase + ?Sized>(&self, _db: &DB, hasher: &mut StableHasher) {
    std::hash::Hasher::write_u16(hasher, *self as u16);
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
      DiagnosticCode::from_repr(tag).unwrap_or_else(|| panic!("unknown DiagnosticCode tag {tag}"));
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

#[cfg(test)]
mod tests {
  use std::fmt::Debug;
  use std::path::PathBuf;

  use proptest::prelude::*;

  use typedown_incremental::{Decodable, Encodable};
  use typedown_incremental::{Decoder, Encoder, QueryStorage};

  use crate::db::{TypedownDatabase, TypedownDecoder, TypedownEncoder};
  use crate::syntax::diagnostic::Diagnostic;
  use crate::syntax::green::{GreenNode, SyntaxToken};
  use crate::syntax::red::RedNode;
  use crate::syntax::syntax_kind::SyntaxKind;

  /// Check that encode and decode return the original value
  fn encode_decode_roundtrip<T: Encodable + Decodable + PartialEq + Debug>(v: &T) {
    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };
    let mut enc = TypedownEncoder::new(&db);
    v.encode(&mut enc);
    let (bytes, intern_blobs) = enc.finish();
    let mut dec = TypedownDecoder::new(&db, &bytes, &intern_blobs);
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
    let (bytes, _) = encoder.finish();

    assert_eq!(bytes, vec![0]);
  }

  #[test]
  fn encode_bool_true_correctly() {
    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };

    let mut encoder = TypedownEncoder::new(&db);
    encoder.emit_bool(true);
    let (bytes, _) = encoder.finish();

    assert_eq!(bytes, vec![1]);
  }

  #[test]
  fn decode_bool_false_correctly() {
    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };

    let data = vec![0];
    let mut decoder = TypedownDecoder::new(&db, &data, &[]);

    let decoded_value = decoder.read_bool();
    assert_eq!(decoded_value, false);
  }

  #[test]
  fn decode_bool_true_correctly() {
    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };

    let data = vec![1];
    let mut decoder = TypedownDecoder::new(&db, &data, &[]);

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

  // String and collection encode/decode

  proptest! {
    #[test]
    fn string_roundtrip(v in any::<String>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn pathbuf_roundtrip(v in any::<PathBuf>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn vec_u8_roundtrip(v in proptest::collection::vec(any::<u8>(), 0..100)) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn vec_i32_roundtrip(v in proptest::collection::vec(any::<i32>(), 0..100)) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn vec_string_roundtrip(v in proptest::collection::vec(any::<String>(), 0..20)) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn option_u32_roundtrip(v in any::<Option<u32>>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn option_string_roundtrip(v in any::<Option<String>>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn vec_option_i64_roundtrip(v in proptest::collection::vec(any::<Option<i64>>(), 0..50)) {
      encode_decode_roundtrip(&v);
    }
  }

  // Compound types

  proptest! {
    #[test]
    fn unit_roundtrip(_v in proptest::strategy::Just(())) {
      encode_decode_roundtrip(&());
    }

    #[test]
    fn tuple1_roundtrip(v in any::<(i32,)>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn tuple2_roundtrip(v in any::<(i32, String)>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn tuple3_roundtrip(v in any::<(u8, bool, String)>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn box_roundtrip(v in any::<Box<i32>>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn either_roundtrip(v in prop_oneof![
      any::<i32>().prop_map(typedown_types::either::Either::<i32, String>::Left),
      any::<String>().prop_map(typedown_types::either::Either::<i32, String>::Right),
    ]) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn hashmap_roundtrip(v in proptest::collection::hash_map(any::<String>(), any::<i32>(), 0..20)) {
      encode_decode_roundtrip(&v);
    }
  }

  // Syntax nodes

  proptest! {
    #[test]
    fn syntax_kind_roundtrip(v in any::<SyntaxKind>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn green_node_roundtrip(v in any::<GreenNode>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn red_node_roundtrip(v in any::<RedNode>()) {
      encode_decode_roundtrip(&v);
    }

    #[test]
    fn diagnostic_roundtrip(v in any::<Diagnostic>()) {
      encode_decode_roundtrip(&v);
    }
  }
}
