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
    debug_assert!(
      self.lex_ctx.mode() == LexMode::YamlFrontmatter,
      "[PeekableLexCtx::peek_yaml] Lex mode must be YamlFrontmatter"
    );

    let saved_after_newline = self.after_newline;
    let saved_yaml_indent = self.yaml_indent;
    let mut skipped_count = 0;

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

    // Rotate the newly appended tokens to the front so they replay in order
    self.token_buffer.rotate_right(skipped_count);

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
    let mut skipped_count = 0;
    let mut found = 0;
    let mut line_offset: usize = 0;

    let result = loop {
      let LexResult { token, diagnostic } = self.lex();
      self.token_buffer.push_back(LexResult {
        token: token.clone(),
        diagnostic: diagnostic.clone(),
      });
      skipped_count += 1;

      // Never peek past EOF
      if token.kind() == SyntaxKind::Eof {
        break LexResult { token, diagnostic };
      }

      let should_skip = match token.kind() {
        SyntaxKind::Whitespace => self.should_skip_ws(skip),
        SyntaxKind::Newline => skip & SKIP_NEWLINE != 0,
        _ => false,
      };
      if !should_skip {
        if found == nth {
          break LexResult { token, diagnostic };
        }
        found += 1;
      }

      // Track characters on current line
      if token.kind() == SyntaxKind::Newline {
        line_offset = 0;
      } else {
        line_offset += token.text_len();
      }
    };

    let prefix_len = line_offset;

    // Rotate the newly appended tokens to the front so they replay in order
    self.token_buffer.rotate_right(skipped_count);

    // Restore state to pre-peek
    self.after_newline = saved_after_newline;

    MdLexResult::new(result)
  }

  /// Whether the most recently lexed token should be skipped given the skip flags.
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
