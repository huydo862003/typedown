//! Expression context for error recovery during parsing.

use typedown_types::syntax_kind::SyntaxKind;

use crate::green::SyntaxToken;

/// Expression context stack entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::parse) enum ExprCtx {
  /// Top-level YAML frontmatter context.
  YamlFrontmatter,
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

  /// Top-level Markdown body context
  MarkdownBody,
  /// Inside a `:::` callout block, closed by `:::`
  /// The `u16` is the indentation level (spaces before `:::`) to append to the prefix.
  MdCalloutBlock(u16),
  /// Inside a `>` blockquote
  MdBlockQuote,
  /// Inside an ordered list (`1. ...`)
  MdOrderedList,
  /// Inside an ordered list item
  MdOrderedListItem,
  /// Inside an unordered list (`- ...`, `* ...`, `+ ...`)
  MdUnorderedList,
  /// Inside an unordered list item
  MdUnorderedListItem,
  /// Inside a toggle list (`>- ...`)
  MdToggleList,
  /// Inside a toggle list item
  MdToggleListItem,

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
}

struct ExprStackEntry {
  ctx: ExprCtx,
  /// Number of prefix tokens this context contributed.
  prefix_token_count: u16,
}

/// Stack of expression contexts for error recovery in expressions.
pub(in crate::parse) struct ExprCtxStack {
  stack: Vec<ExprStackEntry>,
  /// Accumulated MD line prefix as a sequence of expected token kinds.
  md_prefix_tokens: Vec<SyntaxKind>,
}

impl ExprCtxStack {
  pub(in crate::parse) fn new() -> Self {
    Self {
      stack: Vec::new(),
      md_prefix_tokens: Vec::new(),
    }
  }

  /// Push a context onto the stack.
  pub(in crate::parse) fn enter(&mut self, ctx: ExprCtx) {
    let before = self.md_prefix_tokens.len();
    ctx.push_md_prefix_tokens(&mut self.md_prefix_tokens);
    let prefix_token_count = (self.md_prefix_tokens.len() - before) as u16;
    self.stack.push(ExprStackEntry {
      ctx,
      prefix_token_count,
    });
  }

  /// Pop the current context.
  pub(in crate::parse) fn exit(&mut self, expected: ExprCtx) {
    let entry = self.stack.pop();
    debug_assert!(
      entry.as_ref().unwrap().ctx == expected,
      "[ExprCtxStack::exit] Expected {:?} but got {:?}",
      expected,
      entry.as_ref().unwrap().ctx,
    );
    if let Some(entry) = entry {
      let new_len = self.md_prefix_tokens.len() - entry.prefix_token_count as usize;
      self.md_prefix_tokens.truncate(new_len);
    }
  }

  pub(in crate::parse) fn current(&self) -> Option<ExprCtx> {
    self.stack.last().map(|e| e.ctx)
  }

  /// The accumulated expected MD prefix token kinds.
  pub(in crate::parse) fn md_prefix_tokens(&self) -> &[SyntaxKind] {
    &self.md_prefix_tokens
  }

  /// Whether indentation should be ignored.
  /// Return true if any context on the stack ignores indentation (flow constructs)
  pub(in crate::parse) fn should_ignore_indent(&self) -> bool {
    self.stack.iter().any(|e| e.ctx.should_ignore_indent())
  }

  /// Find the innermost context that can handle the given token.
  /// Falls back to the current (innermost) context if none matches.
  pub(in crate::parse) fn find_handler(&self, token: &SyntaxToken) -> Option<ExprCtx> {
    self
      .stack
      .iter()
      .rev()
      .map(|e| e.ctx)
      .find(|ctx| ctx.can_handle(token))
      .or_else(|| self.current())
  }
}

impl ExprCtx {
  pub(in crate::parse) fn is_md_callout_block(self) -> bool {
    matches!(self, ExprCtx::MdCalloutBlock(_))
  }

  /// Push this context's expected prefix tokens.
  fn push_md_prefix_tokens(self, tokens: &mut Vec<SyntaxKind>) {
    match self {
      ExprCtx::MdBlockQuote => {
        tokens.push(SyntaxKind::MdSymbol);
        tokens.push(SyntaxKind::Whitespace);
      }
      ExprCtx::MdUnorderedListItem => {
        tokens.push(SyntaxKind::Whitespace);
      }
      ExprCtx::MdOrderedListItem => {
        tokens.push(SyntaxKind::Whitespace);
      }
      ExprCtx::MdToggleListItem => {
        tokens.push(SyntaxKind::Whitespace);
      }
      ExprCtx::MdCalloutBlock(parent_prefix_count) => {
        if parent_prefix_count > 0 {
          tokens.push(SyntaxKind::Whitespace);
        }
      }
      _ => {}
    }
  }

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
        (ExprCtx::MdCalloutBlock(_), ":::") => true,
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
      | (ExprCtx::MdBlockQuote, SyntaxKind::Newline)
      | (ExprCtx::MdBlockQuote, SyntaxKind::Eof)
      | (ExprCtx::MdOrderedList, SyntaxKind::Eof)
      | (ExprCtx::MdOrderedListItem, SyntaxKind::Newline)
      | (ExprCtx::MdOrderedListItem, SyntaxKind::Eof)
      | (ExprCtx::MdUnorderedList, SyntaxKind::Eof)
      | (ExprCtx::MdUnorderedListItem, SyntaxKind::Newline)
      | (ExprCtx::MdUnorderedListItem, SyntaxKind::Eof)
      | (ExprCtx::MdToggleList, SyntaxKind::Eof)
      | (ExprCtx::MdToggleListItem, SyntaxKind::Newline)
      | (ExprCtx::MdToggleListItem, SyntaxKind::Eof)
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
      | (ExprCtx::MdCalloutBlock(_), SyntaxKind::Eof) => true,
      _ => false,
    }
  }
}
