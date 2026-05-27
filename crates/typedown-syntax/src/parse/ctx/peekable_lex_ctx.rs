use std::{
  collections::VecDeque,
  ops::{Deref, DerefMut},
};

use typedown_types::{stream::Utf8Stream, syntax_kind::SyntaxKind};

use crate::{
  lex::ctx::{LexCtx, LexMode, LexResult},
  parse::constants::{SKIP_COMMENT, SKIP_DEDENT, SKIP_INDENT, SKIP_NEWLINE, SKIP_WS},
};

pub struct YamlLexResult {
  result: LexResult,
  /// Absolute YAML indent depth at the peeked token.
  pub indent: isize,
}

impl YamlLexResult {
  pub(in crate::parse) fn new(result: LexResult, indent: isize) -> Self {
    Self { result, indent }
  }
}

impl std::ops::Deref for YamlLexResult {
  type Target = LexResult;

  fn deref(&self) -> &Self::Target {
    &self.result
  }
}

impl From<YamlLexResult> for LexResult {
  fn from(val: YamlLexResult) -> Self {
    val.result
  }
}

pub struct MdLexResult {
  result: LexResult,
}

impl MdLexResult {
  pub(in crate::parse) fn new(result: LexResult) -> Self {
    Self { result }
  }
}

impl std::ops::Deref for MdLexResult {
  type Target = LexResult;

  fn deref(&self) -> &Self::Target {
    &self.result
  }
}

impl From<MdLexResult> for LexResult {
  fn from(val: MdLexResult) -> Self {
    val.result
  }
}

/// A lex context that allows peeking
/// WARNING: By rewinding, it means that the extracted token can be pushed back to the token stream
/// You CANNOT push back the token's characters, switch mode, re-lex then return a new token
pub struct PeekableLexCtx<S: Utf8Stream> {
  pub(in crate::parse) lex_ctx: LexCtx<S>,
  token_buffer: VecDeque<LexResult>,
  /// Whether the last non-whitespace token was a Newline (or we're at start of input).
  after_newline: bool,
  /// Cumulative YAML indent depth.
  yaml_indent: isize,
}

impl<S: Utf8Stream> PeekableLexCtx<S> {
  pub fn new(lex_ctx: LexCtx<S>) -> PeekableLexCtx<S> {
    PeekableLexCtx {
      lex_ctx,
      token_buffer: VecDeque::default(),
      after_newline: true,
      yaml_indent: 0,
    }
  }

  pub fn yaml_indent(&self) -> isize {
    debug_assert!(
      self.lex_ctx.mode() == LexMode::YamlFrontmatter,
      "[PeekableLexCtx::yaml_indent] Lex mode must be YamlFrontmatter"
    );
    self.yaml_indent
  }

  pub fn lex(&mut self) -> LexResult {
    let front_token = self.token_buffer.pop_front();
    let result = match front_token {
      Some(token) => token,
      None => self.lex_ctx.lex(),
    };

    // Track YAML indent depth
    match result.token.kind() {
      SyntaxKind::YamlIndent => self.yaml_indent += 1,
      SyntaxKind::YamlDedent => self.yaml_indent -= 1,
      _ => {}
    }

    // Track line position
    match result.token.kind() {
      SyntaxKind::Newline => self.after_newline = true,
      SyntaxKind::Whitespace => {} // whitespace doesn't change line position
      _ => self.after_newline = false,
    }

    result
  }

  pub fn peek(&mut self, skip: u16, mode: LexMode) -> LexResult {
    debug_assert!(
      self.lex_ctx.mode() == mode,
      "[PeekableLexCtx::peek] Lex mode must be the same as the `mode` argument"
    );
    match mode {
      LexMode::YamlFrontmatter => self.peek_yaml(skip).into(),
      LexMode::MarkdownBody => self.peek_md(skip).into(),
    }
  }

  /// Peek at the next non-skipped YAML token without consuming.
  pub fn peek_yaml(&mut self, skip: u16) -> YamlLexResult {
    self.peek_yaml_nth(0, skip)
  }

  /// Peek at the nth non-skipped YAML token without consuming (0-indexed).
  pub fn peek_yaml_nth(&mut self, nth: usize, skip: u16) -> YamlLexResult {
    debug_assert!(
      self.lex_ctx.mode() == LexMode::YamlFrontmatter,
      "[PeekableLexCtx::peek_yaml_nth] Lex mode must be YamlFrontmatter"
    );

    let saved_after_newline = self.after_newline;
    let saved_yaml_indent = self.yaml_indent;
    let mut prefetch: VecDeque<LexResult> = VecDeque::new();
    let mut found = 0;

    let result = loop {
      let lex_result = self.lex();
      let token = lex_result.token.clone();
      let diagnostic = lex_result.diagnostic.clone();
      prefetch.push_back(lex_result);

      if token.kind() == SyntaxKind::Eof {
        break LexResult { token, diagnostic };
      }
      if token.kind() == SyntaxKind::YamlOp && token.text().collect::<String>() == "---" {
        break LexResult { token, diagnostic };
      }

      if !self.should_skip(token.kind(), skip) {
        if found == nth {
          break LexResult { token, diagnostic };
        }
        found += 1;
      }
    };

    for token in prefetch.into_iter().rev() {
      self.token_buffer.push_front(token);
    }
    let peeked_indent = self.yaml_indent;

    // Restore state to pre-peek
    self.after_newline = saved_after_newline;
    self.yaml_indent = saved_yaml_indent;

    YamlLexResult::new(result, peeked_indent)
  }

  /// Peek at the next non-skipped Markdown token without consuming.
  pub fn peek_md(&mut self, skip: u16) -> MdLexResult {
    self.peek_md_nth(0, skip)
  }

  /// Peek at the nth non-skipped Markdown token without consuming.
  pub fn peek_md_nth(&mut self, nth: usize, skip: u16) -> MdLexResult {
    debug_assert!(
      self.lex_ctx.mode() == LexMode::MarkdownBody,
      "[PeekableLexCtx::peek_md_nth] Lex mode must be MarkdownBody"
    );
    let saved_after_newline = self.after_newline;
    let mut prefetch: VecDeque<LexResult> = VecDeque::new();
    let mut found = 0;

    let result = loop {
      let lex_result = self.lex();
      let token = lex_result.token.clone();
      let diagnostic = lex_result.diagnostic.clone();
      prefetch.push_back(lex_result);

      if token.kind() == SyntaxKind::Eof {
        break LexResult { token, diagnostic };
      }

      if !self.should_skip(token.kind(), skip) {
        if found == nth {
          break LexResult { token, diagnostic };
        }
        found += 1;
      }
    };

    for token in prefetch.into_iter().rev() {
      self.token_buffer.push_front(token);
    }

    // Restore state to pre-peek
    self.after_newline = saved_after_newline;

    MdLexResult::new(result)
  }

  /// Whether the most recently lexed token should be skipped given the skip flags.
  pub fn should_skip(&self, kind: SyntaxKind, skip: u16) -> bool {
    match kind {
      SyntaxKind::Whitespace => skip & SKIP_WS != 0,
      SyntaxKind::YamlComment => skip & SKIP_COMMENT != 0,
      SyntaxKind::Newline => skip & SKIP_NEWLINE != 0,
      SyntaxKind::YamlIndent => skip & SKIP_INDENT != 0,
      SyntaxKind::YamlDedent => skip & SKIP_DEDENT != 0,
      _ => false,
    }
  }
}

impl<S: Utf8Stream> Deref for PeekableLexCtx<S> {
  type Target = LexCtx<S>;

  fn deref(&self) -> &Self::Target {
    &self.lex_ctx
  }
}

impl<S: Utf8Stream> DerefMut for PeekableLexCtx<S> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.lex_ctx
  }
}
