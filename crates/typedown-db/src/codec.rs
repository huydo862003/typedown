use std::collections::HashMap;

use typedown_syntax::green::GreenNode;
use typedown_syntax::green::cache::with_green_cache;
use typedown_syntax::red::RedNode;
use typedown_types::syntax_kind::SyntaxKind;

use crate::{Decodable, Decoder, Encodable, Encoder, QueryDatabase, TypedownDatabase};

// TypedownEncoder
pub struct TypedownEncoder<'a> {
  pub db: &'a TypedownDatabase,
  pub buf: Vec<u8>,
  pub intern_hints: HashMap<usize, u32>,
  pub intern_table: HashMap<Vec<u8>, u32>,
  pub intern_blobs: Vec<Vec<u8>>,
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

  fn emit_u8(&mut self, v: u8) {
    self.buf.push(v);
  }

  fn emit_u16(&mut self, v: u16) {
    self.buf.extend_from_slice(&v.to_le_bytes());
  }

  fn emit_u32(&mut self, v: u32) {
    self.buf.extend_from_slice(&v.to_le_bytes());
  }

  fn emit_u64(&mut self, v: u64) {
    self.buf.extend_from_slice(&v.to_le_bytes());
  }

  fn emit_usize(&mut self, v: usize) {
    self.emit_u64(v as u64);
  }

  fn emit_i8(&mut self, v: i8) {
    self.buf.push(v as u8);
  }

  fn emit_i32(&mut self, v: i32) {
    self.buf.extend_from_slice(&v.to_le_bytes());
  }

  fn emit_i64(&mut self, v: i64) {
    self.buf.extend_from_slice(&v.to_le_bytes());
  }

  fn emit_f64(&mut self, v: f64) {
    self.buf.extend_from_slice(&v.to_le_bytes());
  }

  fn emit_bool(&mut self, v: bool) {
    self.buf.push(v as u8);
  }

  fn emit_char(&mut self, v: char) {
    self.emit_u32(v as u32);
  }

  fn emit_str(&mut self, v: &str) {
    self.emit_u32(v.len() as u32);
    self.buf.extend_from_slice(v.as_bytes());
  }

  fn emit_bytes(&mut self, v: &[u8]) {
    self.emit_u32(v.len() as u32);
    self.buf.extend_from_slice(v);
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

  fn read_bytes(&mut self, n: usize) -> &'a [u8] {
    let bytes = &self.data[self.pos..self.pos + n];
    self.pos += n;
    bytes
  }
}

impl Decoder for TypedownDecoder<'_> {
  fn db(&self) -> &dyn QueryDatabase {
    self.db
  }

  fn read_u8(&mut self) -> u8 {
    let v = self.data[self.pos];
    self.pos += 1;
    v
  }

  fn read_u16(&mut self) -> u16 {
    u16::from_le_bytes(self.read_bytes(2).try_into().unwrap())
  }

  fn read_u32(&mut self) -> u32 {
    u32::from_le_bytes(self.read_bytes(4).try_into().unwrap())
  }

  fn read_u64(&mut self) -> u64 {
    u64::from_le_bytes(self.read_bytes(8).try_into().unwrap())
  }

  fn read_usize(&mut self) -> usize {
    self.read_u64() as usize
  }

  fn read_i8(&mut self) -> i8 {
    self.read_u8() as i8
  }

  fn read_i32(&mut self) -> i32 {
    i32::from_le_bytes(self.read_bytes(4).try_into().unwrap())
  }

  fn read_i64(&mut self) -> i64 {
    i64::from_le_bytes(self.read_bytes(8).try_into().unwrap())
  }

  fn read_f64(&mut self) -> f64 {
    f64::from_le_bytes(self.read_bytes(8).try_into().unwrap())
  }

  fn read_bool(&mut self) -> bool {
    self.read_u8() != 0
  }

  fn read_char(&mut self) -> char {
    char::from_u32(self.read_u32()).unwrap()
  }

  fn read_str(&mut self) -> String {
    let len = self.read_u32() as usize;
    let bytes = self.read_bytes(len);
    String::from_utf8(bytes.to_vec()).unwrap()
  }

  fn read_bytes_owned(&mut self) -> Vec<u8> {
    let len = self.read_u32() as usize;
    self.read_bytes(len).to_vec()
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

// Diagnostic
use typedown_types::diagnostic::{Diagnostic, DiagnosticCode};

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
