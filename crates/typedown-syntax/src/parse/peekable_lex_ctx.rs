use std::{
  collections::VecDeque,
  ops::{Deref, DerefMut},
};

use typedown_types::{stream::Utf8Stream, syntax_kind::SyntaxKind};

use crate::{
  lex::ctx::{LexCtx, LexMode, LexResult},
  parse::constants::{
    SKIP_COMMENT, SKIP_DEDENT, SKIP_INDENT, SKIP_LEADING_WS, SKIP_MIDDLE_WS, SKIP_NEWLINE,
    SKIP_STANDALONE_WS, SKIP_TRAILING_WS, SKIP_WS,
  },
};

/// Result of `peek_yaml`, wrapping a `LexResult` with the indent depth at the peeked token.
pub struct PeekYamlResult(pub LexResult, pub usize);

impl std::ops::Deref for PeekYamlResult {
  type Target = LexResult;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

/// A lex context that allows peeking
/// WARNING: By rewinding, it means that the extracted token can be pushed back to the token stream
/// You CANNOT push back the token's characters, switch mode, re-lex then return a new token
pub struct PeekableLexCtx<S: Utf8Stream> {
  pub(super) lex_ctx: LexCtx<S>,
  token_buffer: VecDeque<LexResult>,
  /// Current YAML indent depth.
  indent_depth: usize,
  /// Whether the last non-whitespace token was a Newline (or we're at start of input).
  after_newline: bool,
}

impl<S: Utf8Stream> PeekableLexCtx<S> {
  pub fn new(lex_ctx: LexCtx<S>) -> PeekableLexCtx<S> {
    PeekableLexCtx {
      lex_ctx,
      token_buffer: VecDeque::default(),
      indent_depth: 0,
      after_newline: true,
    }
  }

  pub fn indent_depth(&self) -> usize {
    self.indent_depth
  }

  pub fn lex(&mut self) -> LexResult {
    let front_token = self.token_buffer.pop_front();
    let result = match front_token {
      Some(token) => token,
      None => self.lex_ctx.lex(),
    };

    // Track indent depth
    match result.token.kind() {
      SyntaxKind::YamlIndent => self.indent_depth += 1,
      SyntaxKind::YamlDedent => {
        self.indent_depth = self.indent_depth.saturating_sub(1);
      }
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
      LexMode::YamlFrontmatter => self.peek_yaml(skip).0,
      LexMode::MarkdownBody => self.peek_md(skip),
    }
  }

  /// Peek at the next non-skipped YAML token without consuming.
  pub fn peek_yaml(&mut self, skip: u16) -> PeekYamlResult {
    debug_assert!(
      self.lex_ctx.mode() == LexMode::YamlFrontmatter,
      "[PeekableLexCtx::peek_yaml] Lex mode must be YamlFrontmatter"
    );

    // Saved state before peeking
    let saved_indent_depth = self.indent_depth;
    let saved_after_newline = self.after_newline;

    let mut skipped_count = 0; // Used for rotating the token_buffer

    let result = loop {
      let LexResult { token, diagnostic } = self.lex();
      self.token_buffer.push_back(LexResult {
        token: token.clone(),
        diagnostic: diagnostic.clone(),
      });
      skipped_count += 1;

      // Never peek past --- or EOF
      if token.kind() == SyntaxKind::Eof {
        break LexResult { token, diagnostic };
      }
      if token.kind() == SyntaxKind::YamlOp && token.text().collect::<String>() == "---" {
        break LexResult { token, diagnostic };
      }

      let should_skip = match token.kind() {
        SyntaxKind::Whitespace => self.should_skip_ws(skip),
        SyntaxKind::YamlComment => skip & SKIP_COMMENT != 0,
        SyntaxKind::Newline => skip & SKIP_NEWLINE != 0,
        SyntaxKind::YamlIndent => skip & SKIP_INDENT != 0,
        SyntaxKind::YamlDedent => skip & SKIP_DEDENT != 0,
        _ => false,
      };

      if !should_skip {
        break LexResult { token, diagnostic };
      }
    };

    let peeked_indent_depth = self.indent_depth;

    // Rotate the newly appended tokens to the front so they replay in order
    self.token_buffer.rotate_right(skipped_count);

    // Restore state to pre-peek
    self.indent_depth = saved_indent_depth;
    self.after_newline = saved_after_newline;

    PeekYamlResult(result, peeked_indent_depth)
  }

  /// Peek at the next non-skipped Markdown token without consuming.
  pub fn peek_md(&mut self, skip: u16) -> LexResult {
    debug_assert!(
      self.lex_ctx.mode() == LexMode::MarkdownBody,
      "[PeekableLexCtx::peek_md] Lex mode must be MarkdownBody"
    );
    let mut skipped_count = 0;

    let result = loop {
      let LexResult { token, diagnostic } = self.lex();
      self.token_buffer.push_back(LexResult {
        token: token.clone(),
        diagnostic: diagnostic.clone(),
      });
      skipped_count += 1;

      let should_skip = match token.kind() {
        SyntaxKind::Whitespace => skip & SKIP_WS != 0,
        SyntaxKind::Newline => skip & SKIP_NEWLINE != 0,
        _ => false,
      };
      if !should_skip {
        break LexResult { token, diagnostic };
      }
    };

    // Rotate the newly appended tokens to the front so they replay in order
    self.token_buffer.rotate_right(skipped_count);

    result
  }

  /// Whether the most recently lexed token should be skipped in YAML mode.
  pub fn should_skip(&mut self, kind: SyntaxKind, skip: u16) -> bool {
    match kind {
      SyntaxKind::Whitespace => self.should_skip_ws(skip),
      SyntaxKind::YamlComment => skip & SKIP_COMMENT != 0,
      SyntaxKind::Newline => skip & SKIP_NEWLINE != 0,
      SyntaxKind::YamlIndent => skip & SKIP_INDENT != 0,
      SyntaxKind::YamlDedent => skip & SKIP_DEDENT != 0,
      _ => false,
    }
  }

  /// Peek at the kind of the very next token without consuming or skipping.
  fn peek_next_kind(&mut self) -> SyntaxKind {
    if let Some(front) = self.token_buffer.front() {
      return front.token.kind();
    }
    let result = self.lex_ctx.lex();
    let kind = result.token.kind();
    self.token_buffer.push_back(result);
    kind
  }

  /// Check whether a whitespace token should be skipped given the skip flags.
  fn should_skip_ws(&mut self, skip: u16) -> bool {
    if skip & SKIP_WS == SKIP_WS {
      return true;
    }
    let next_kind = self.peek_next_kind();
    let before_newline = matches!(next_kind, SyntaxKind::Newline | SyntaxKind::Eof);
    match (self.after_newline, before_newline) {
      (true, true) => skip & SKIP_STANDALONE_WS != 0,
      (true, false) => skip & SKIP_LEADING_WS != 0,
      (false, true) => skip & SKIP_TRAILING_WS != 0,
      (false, false) => skip & SKIP_MIDDLE_WS != 0,
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
