//! Expression context for error recovery during parsing.

use typedown_types::syntax_kind::SyntaxKind;

/// Expression context stack entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::parse) enum ExprCtx {
  /// Inside `${...}` interpolation, closed by `}`
  Interp,
  /// Inside `[...]` list, closed by `]`
  List,
  /// Inside `{...}` dict, closed by `}`
  Dict,
  /// Inside `"..."` double-quoted string, closed by `"`
  DqString,
  /// Inside `'...'` single-quoted string, closed by `'`
  SqString,
  /// Inside `(...)` parenthesized expression, closed by `)`
  Paren,
  /// Inside `func(...)` call expression, closed by `)`
  Call,
}

/// Stack of expression contexts for error recovery in expressions.
pub(in crate::parse) struct ExprCtxStack(Vec<ExprCtx>);

impl ExprCtxStack {
  pub(in crate::parse) fn new() -> Self {
    Self(Vec::new())
  }

  /// Push a context onto the stack.
  pub(in crate::parse) fn enter(&mut self, ctx: ExprCtx) {
    self.0.push(ctx);
  }

  /// Pop the current context.
  pub(in crate::parse) fn exit(&mut self, expected: ExprCtx) {
    let popped = self.0.pop();
    debug_assert!(
      popped == Some(expected),
      "[ExprCtxStack::exit] Expected {:?} but got {:?}",
      expected,
      popped
    );
  }

  pub(in crate::parse) fn current(&self) -> Option<ExprCtx> {
    self.0.last().copied()
  }

  /// Whether indentation should be ignored.
  /// True if any context on the stack ignores indentation (flow constructs).
  pub(in crate::parse) fn should_ignore_indent(&self) -> bool {
    self.0.iter().any(|ctx| ctx.should_ignore_indent())
  }

  /// Find the innermost context that can handle the given token kind.
  /// Falls back to the current (innermost) context if none matches.
  pub(in crate::parse) fn find_handler(&self, kind: SyntaxKind) -> Option<ExprCtx> {
    self
      .0
      .iter()
      .rev()
      .copied()
      .find(|ctx| ctx.can_handle(kind))
      .or_else(|| self.current())
  }
}

impl ExprCtx {
  /// Whether indentation is irrelevant in this context (flow constructs).
  pub(in crate::parse) fn should_ignore_indent(self) -> bool {
    matches!(self, ExprCtx::List | ExprCtx::Dict | ExprCtx::Paren | ExprCtx::Call)
  }

  /// Whether this context can handle the given token kind.
  pub(in crate::parse) fn can_handle(self, kind: SyntaxKind) -> bool {
    matches!(
      (self, kind),
      (ExprCtx::Interp, SyntaxKind::InterpEnd)
        | (ExprCtx::List, SyntaxKind::RBracket)
        | (ExprCtx::List, SyntaxKind::Comma)
        | (ExprCtx::Dict, SyntaxKind::RBrace)
        | (ExprCtx::Dict, SyntaxKind::Comma)
        | (ExprCtx::Dict, SyntaxKind::Colon)
        | (ExprCtx::DqString, SyntaxKind::DqStrEnd)
        | (ExprCtx::SqString, SyntaxKind::SqStrEnd)
        | (ExprCtx::Paren, SyntaxKind::RParen)
        | (ExprCtx::Call, SyntaxKind::RParen)
        | (ExprCtx::Call, SyntaxKind::Comma)
    )
  }
}
