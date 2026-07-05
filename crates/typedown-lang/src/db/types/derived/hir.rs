use crate::syntax::red::RedNode;
use num_enum::TryFromPrimitive;
use typedown_macros::query_derived;
use typedown_types::diagnostic::Diagnostic;

use crate::db::types::{File, Project};
use typedown_incremental::{Decodable, Decoder, Encodable, Encoder, StableHash, StableHasher};

/// A lowered YAML value, source-tracked via its originating project, file, and red node.
#[query_derived]
pub struct HirValue {
  #[id]
  pub project: Project,
  #[id]
  pub file: File,
  #[id]
  pub node: RedNode,
  pub kind: HirValueKind,
  pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum HirValueKind {
  Str(String),
  Num(String),
  Math(String),
  Bool(bool),
  Null,
  Ident(String),
  Mapping(Vec<(String, HirValue)>),
  Sequence(Vec<HirValue>),
  Interpolated(Vec<InterpolatedPart>),
  Markdown(Vec<InterpolatedPart>),
  Tag {
    tag: Box<HirValue>,
    inner: Box<HirValue>,
  },
  Unary {
    op: String,
    operand: Box<HirValue>,
  },
  Binary {
    op: String,
    left: Box<HirValue>,
    right: Box<HirValue>,
  },
  Call {
    callee: Box<HirValue>,
    args: Vec<HirValue>,
  },
  Index {
    expr: Box<HirValue>,
    indices: Vec<HirValue>,
  },
}

impl StableHash for HirValueKind {
  fn stable_hash<DB: ::typedown_incremental::QueryDatabase + ?Sized>(
    &self,
    db: &DB,
    hasher: &mut StableHasher,
  ) {
    std::mem::discriminant(self).stable_hash(db, hasher);
    match self {
      HirValueKind::Str(v)
      | HirValueKind::Num(v)
      | HirValueKind::Math(v)
      | HirValueKind::Ident(v) => v.stable_hash(db, hasher),
      HirValueKind::Bool(v) => v.stable_hash(db, hasher),
      HirValueKind::Null => {}
      HirValueKind::Mapping(entries) => entries.stable_hash(db, hasher),
      HirValueKind::Sequence(items) => items.stable_hash(db, hasher),
      HirValueKind::Interpolated(parts) | HirValueKind::Markdown(parts) => {
        parts.stable_hash(db, hasher)
      }
      HirValueKind::Tag { tag, inner } => {
        tag.stable_hash(db, hasher);
        inner.stable_hash(db, hasher);
      }
      HirValueKind::Unary { op, operand } => {
        op.stable_hash(db, hasher);
        operand.stable_hash(db, hasher);
      }
      HirValueKind::Binary { op, left, right } => {
        op.stable_hash(db, hasher);
        left.stable_hash(db, hasher);
        right.stable_hash(db, hasher);
      }
      HirValueKind::Call { callee, args } => {
        callee.stable_hash(db, hasher);
        args.stable_hash(db, hasher);
      }
      HirValueKind::Index { expr, indices } => {
        expr.stable_hash(db, hasher);
        indices.stable_hash(db, hasher);
      }
    }
  }
}

impl StableHash for InterpolatedPart {
  fn stable_hash<DB: ::typedown_incremental::QueryDatabase + ?Sized>(
    &self,
    db: &DB,
    hasher: &mut StableHasher,
  ) {
    std::mem::discriminant(self).stable_hash(db, hasher);
    match self {
      InterpolatedPart::Literal(s) => s.stable_hash(db, hasher),
      InterpolatedPart::Expr(hir) => hir.stable_hash(db, hasher),
    }
  }
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
enum HirValueKindTag {
  Str = 0,
  Num = 1,
  Math = 2,
  Bool = 3,
  Null = 4,
  Ident = 5,
  Mapping = 6,
  Sequence = 7,
  Interpolated = 8,
  Markdown = 9,
  Tag = 10,
  Unary = 11,
  Binary = 12,
  Call = 13,
  Index = 14,
}

#[derive(TryFromPrimitive)]
#[repr(u8)]
enum InterpolatedPartTag {
  Literal = 0,
  Expr = 1,
}

impl Encodable for HirValueKind {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    match self {
      HirValueKind::Str(val) => {
        encoder.emit_u8(HirValueKindTag::Str as u8);
        val.encode(encoder);
      }
      HirValueKind::Num(val) => {
        encoder.emit_u8(HirValueKindTag::Num as u8);
        val.encode(encoder);
      }
      HirValueKind::Math(val) => {
        encoder.emit_u8(HirValueKindTag::Math as u8);
        val.encode(encoder);
      }
      HirValueKind::Bool(val) => {
        encoder.emit_u8(HirValueKindTag::Bool as u8);
        val.encode(encoder);
      }
      HirValueKind::Null => {
        encoder.emit_u8(HirValueKindTag::Null as u8);
      }
      HirValueKind::Ident(val) => {
        encoder.emit_u8(HirValueKindTag::Ident as u8);
        val.encode(encoder);
      }
      HirValueKind::Mapping(entries) => {
        encoder.emit_u8(HirValueKindTag::Mapping as u8);
        entries.encode(encoder);
      }
      HirValueKind::Sequence(items) => {
        encoder.emit_u8(HirValueKindTag::Sequence as u8);
        items.encode(encoder);
      }
      HirValueKind::Interpolated(parts) => {
        encoder.emit_u8(HirValueKindTag::Interpolated as u8);
        parts.encode(encoder);
      }
      HirValueKind::Markdown(parts) => {
        encoder.emit_u8(HirValueKindTag::Markdown as u8);
        parts.encode(encoder);
      }
      HirValueKind::Tag { tag, inner } => {
        encoder.emit_u8(HirValueKindTag::Tag as u8);
        tag.encode(encoder);
        inner.encode(encoder);
      }
      HirValueKind::Unary { op, operand } => {
        encoder.emit_u8(HirValueKindTag::Unary as u8);
        op.encode(encoder);
        operand.encode(encoder);
      }
      HirValueKind::Binary { op, left, right } => {
        encoder.emit_u8(HirValueKindTag::Binary as u8);
        op.encode(encoder);
        left.encode(encoder);
        right.encode(encoder);
      }
      HirValueKind::Call { callee, args } => {
        encoder.emit_u8(HirValueKindTag::Call as u8);
        callee.encode(encoder);
        args.encode(encoder);
      }
      HirValueKind::Index { expr, indices } => {
        encoder.emit_u8(HirValueKindTag::Index as u8);
        expr.encode(encoder);
        indices.encode(encoder);
      }
    }
  }
}

impl Decodable for HirValueKind {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    let tag = decoder.read_u8();
    match HirValueKindTag::try_from(tag)
      .unwrap_or_else(|_| panic!("unknown HirValueKind tag {tag}"))
    {
      HirValueKindTag::Str => HirValueKind::Str(String::decode(decoder)),
      HirValueKindTag::Num => HirValueKind::Num(String::decode(decoder)),
      HirValueKindTag::Math => HirValueKind::Math(String::decode(decoder)),
      HirValueKindTag::Bool => HirValueKind::Bool(bool::decode(decoder)),
      HirValueKindTag::Null => HirValueKind::Null,
      HirValueKindTag::Ident => HirValueKind::Ident(String::decode(decoder)),
      HirValueKindTag::Mapping => HirValueKind::Mapping(Vec::decode(decoder)),
      HirValueKindTag::Sequence => HirValueKind::Sequence(Vec::decode(decoder)),
      HirValueKindTag::Interpolated => HirValueKind::Interpolated(Vec::decode(decoder)),
      HirValueKindTag::Markdown => HirValueKind::Markdown(Vec::decode(decoder)),
      HirValueKindTag::Tag => HirValueKind::Tag {
        tag: Box::decode(decoder),
        inner: Box::decode(decoder),
      },
      HirValueKindTag::Unary => HirValueKind::Unary {
        op: String::decode(decoder),
        operand: Box::decode(decoder),
      },
      HirValueKindTag::Binary => HirValueKind::Binary {
        op: String::decode(decoder),
        left: Box::decode(decoder),
        right: Box::decode(decoder),
      },
      HirValueKindTag::Call => HirValueKind::Call {
        callee: Box::decode(decoder),
        args: Vec::decode(decoder),
      },
      HirValueKindTag::Index => HirValueKind::Index {
        expr: Box::decode(decoder),
        indices: Vec::decode(decoder),
      },
    }
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum InterpolatedPart {
  Literal(String),
  Expr(HirValue),
}

impl Encodable for InterpolatedPart {
  fn encode<E: Encoder + ?Sized>(&self, encoder: &mut E) {
    match self {
      InterpolatedPart::Literal(s) => {
        encoder.emit_u8(InterpolatedPartTag::Literal as u8);
        s.encode(encoder);
      }
      InterpolatedPart::Expr(hir) => {
        encoder.emit_u8(InterpolatedPartTag::Expr as u8);
        hir.encode(encoder);
      }
    }
  }
}

impl Decodable for InterpolatedPart {
  fn decode<D: Decoder + ?Sized>(decoder: &mut D) -> Self {
    let tag = decoder.read_u8();
    match InterpolatedPartTag::try_from(tag)
      .unwrap_or_else(|_| panic!("unknown InterpolatedPart tag {tag}"))
    {
      InterpolatedPartTag::Literal => InterpolatedPart::Literal(String::decode(decoder)),
      InterpolatedPartTag::Expr => InterpolatedPart::Expr(HirValue::decode(decoder)),
    }
  }
}
