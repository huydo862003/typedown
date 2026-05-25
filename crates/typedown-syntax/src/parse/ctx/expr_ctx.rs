//! Expression context for error recovery during parsing.

use typedown_types::syntax_kind::SyntaxKind;

use crate::green::SyntaxToken;

/// Expression context stack entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::parse) enum ExprCtx {
  /// Top-level YAML frontmatter context.
  YamlFrontmatter,
  /// Top-level Markdown body context
  MarkdownBody,
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
  /// Inside a block sequence
  BlockSeq,
  /// Inside a block mapping
  BlockMap,
  /// Inside a markdown list item at the given indentation level
  MdListItem(usize),
  /// Inside the text part of a markdown link or media: `[text]`
  MdLinkText,
  /// Inside the url/src part of a markdown link or media: `(url)`
  MdLinkUrl,
  /// Inside `**text**`, closed by `**`
  MdBold,
  /// Inside `*text*`, closed by `*`
  MdItalicStar,
  /// Inside `_text_`, closed by `_`
  MdItalicUnderscore,
  /// Inside `***text***`, closed by `***`
  MdBoldItalic,
  /// Inside `~~text~~`, closed by `~~`
  MdStrikethrough,
  /// Inside `[@key]`, closed by `]`
  MdCitation,
  /// Inside a `:::` callout block, closed by `:::`
  MdCalloutBlock,
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
  /// Return true if any context on the stack ignores indentation (flow constructs)
  pub(in crate::parse) fn should_ignore_indent(&self) -> bool {
    self.0.iter().any(|ctx| ctx.should_ignore_indent())
  }

  /// Find the innermost context that can handle the given token.
  /// Falls back to the current (innermost) context if none matches.
  pub(in crate::parse) fn find_handler(&self, token: &SyntaxToken) -> Option<ExprCtx> {
    self
      .0
      .iter()
      .rev()
      .copied()
      .find(|ctx| ctx.can_handle(token))
      .or_else(|| self.current())
  }
}

impl ExprCtx {
  /// Whether indentation is irrelevant in this context (flow constructs).
  pub(in crate::parse) fn should_ignore_indent(self) -> bool {
    matches!(
      self,
      ExprCtx::List | ExprCtx::Dict | ExprCtx::Paren | ExprCtx::Call
    )
  }

  /// Whether this context can handle the given token.
  pub(in crate::parse) fn can_handle(self, token: &SyntaxToken) -> bool {
    // Text-dependent checks for MdSymbol closing delimiters
    if token.kind() == SyntaxKind::MdSymbol {
      let text: String = token.text().collect();
      return match (self, text.as_str()) {
        (ExprCtx::YamlFrontmatter, "---") => true,
        (ExprCtx::MdBold, "**") => true,
        (ExprCtx::MdItalicStar, "*") => true,
        (ExprCtx::MdItalicUnderscore, "_") => true,
        (ExprCtx::MdBoldItalic, "***") => true,
        (ExprCtx::MdStrikethrough, "~~") => true,
        (ExprCtx::MdCalloutBlock, ":::") => true,
        _ => false,
      };
    }

    match (self, token.kind()) {
      (ExprCtx::YamlFrontmatter, SyntaxKind::YamlIndent)
      | (ExprCtx::YamlFrontmatter, SyntaxKind::YamlDedent)
      | (ExprCtx::YamlFrontmatter, SyntaxKind::Eof)
      | (ExprCtx::MarkdownBody, SyntaxKind::Eof)
      | (ExprCtx::Interp, SyntaxKind::InterpEnd)
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
      | (ExprCtx::BlockSeq, SyntaxKind::Newline)
      | (ExprCtx::BlockSeq, SyntaxKind::YamlDedent)
      | (ExprCtx::BlockMap, SyntaxKind::Newline)
      | (ExprCtx::BlockMap, SyntaxKind::YamlDedent)
      | (ExprCtx::MdListItem(_), SyntaxKind::Newline)
      | (ExprCtx::MdListItem(_), SyntaxKind::Eof)
      | (ExprCtx::MdLinkText, SyntaxKind::RBracket)
      | (ExprCtx::MdLinkText, SyntaxKind::Newline)
      | (ExprCtx::MdLinkText, SyntaxKind::Eof)
      | (ExprCtx::MdLinkUrl, SyntaxKind::RParen)
      | (ExprCtx::MdLinkUrl, SyntaxKind::Newline)
      | (ExprCtx::MdLinkUrl, SyntaxKind::Eof)
      | (ExprCtx::MdBold, SyntaxKind::Newline)
      | (ExprCtx::MdBold, SyntaxKind::Eof)
      | (ExprCtx::MdItalicStar, SyntaxKind::Newline)
      | (ExprCtx::MdItalicStar, SyntaxKind::Eof)
      | (ExprCtx::MdItalicUnderscore, SyntaxKind::Newline)
      | (ExprCtx::MdItalicUnderscore, SyntaxKind::Eof)
      | (ExprCtx::MdBoldItalic, SyntaxKind::Newline)
      | (ExprCtx::MdBoldItalic, SyntaxKind::Eof)
      | (ExprCtx::MdStrikethrough, SyntaxKind::Newline)
      | (ExprCtx::MdStrikethrough, SyntaxKind::Eof)
      | (ExprCtx::MdCitation, SyntaxKind::RBracket)
      | (ExprCtx::MdCitation, SyntaxKind::Newline)
      | (ExprCtx::MdCitation, SyntaxKind::Eof)
      | (ExprCtx::MdCalloutBlock, SyntaxKind::Eof) => true,
      _ => false,
    }
  }
}
