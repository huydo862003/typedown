use typedown_macros::query_derived;
use typedown_syntax::red::RedNode;
use typedown_types::diagnostic::Diagnostic;

use crate::types::{File, Project};

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

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum InterpolatedPart {
  Literal(String),
  Expr(HirValue),
}
