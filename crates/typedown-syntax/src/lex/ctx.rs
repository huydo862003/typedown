//! An on-demand lexer
//! Supports 2 lex modes:
//! - YAML frontmatter mode
//! - Markdown mode
//! The parser will trigger lex and switch mode where sensible
//! This produces one green token at a time

use std::cell::RefCell;
use std::rc::Rc;

use typedown_types::stream::{Utf8Result, Utf8Stream};

use crate::green::cache::Cache;
use crate::green::syntax_kind::SyntaxKind;
use crate::green::token::Token;
use crate::lex::diagnostic::LexDiagnostic;

pub struct LexResult {
  pub token: Token,
  pub diagnostic: Option<LexDiagnostic>,
}

pub enum LexMode {
  YamlFrontmatter,
  MarkdownBody,
}

/// An on-demand lexer supporting 2 lex modes
pub struct LexCtx<S: Utf8Stream> {
  stream: S,
  cache: Rc<RefCell<Cache>>,
  // Current mode
  mode: LexMode,
  // Text buffer to accumulate the read utf-8
  text_buffer: String,
}

impl<S: Utf8Stream> LexCtx<S> {
  pub fn new(stream: S, cache: Rc<RefCell<Cache>>) -> Self {
    Self {
      stream,
      cache,
      mode: LexMode::YamlFrontmatter,
      text_buffer: String::from(""),
    }
  }

  /// Switch the lexing mode.
  pub fn set_mode(&mut self, mode: LexMode) {
    self.mode = mode;
  }

  /// Lex the next token based on the current mode.
  pub fn lex(&mut self) -> LexResult {
    if self.is_eof() {
      self.emit(SyntaxKind::Eof)
    } else {
      let maybe_invalid_utf8 = self.try_consume_invalid_utf8();
      if let Some(result) = maybe_invalid_utf8 {
        return result;
      }
      match self.mode {
        LexMode::YamlFrontmatter => self.lex_frontmatter(),
        LexMode::MarkdownBody => self.lex_body(),
      }
    }
  }
}

// YAML frontmatter lexing
impl<S: Utf8Stream> LexCtx<S> {
  fn lex_frontmatter(&mut self) -> LexResult {
    todo!()
  }
}

// Markdown body lexing
impl<S: Utf8Stream> LexCtx<S> {
  fn lex_body(&mut self) -> LexResult {
    todo!()
  }
}

// Shared helpers
impl<S: Utf8Stream> LexCtx<S> {
  /// Look at the next character without consuming it.
  fn peek(&mut self) -> Utf8Result {
    self.stream.peek()
  }

  /// Consume the next character, appending it to the current token text.
  /// If invalid UTF-8 is encountered, do not consume it and return the result.
  fn advance_avoid_invalid_utf8(&mut self) -> Utf8Result {
    match self.stream.peek() {
      Utf8Result::Char(_) => {
        let result = self.stream.advance();
        if let Utf8Result::Char(char) = result {
          self.text_buffer.push(char);
        }
        result
      }
      other => other,
    }
  }

  /// Consume the next character if it matches `expected`.
  fn consume_avoid_invalid_utf8(&mut self, expected: char) -> bool {
    if let Utf8Result::Char(encountered) = self.peek()
      && encountered == expected
    {
      self.advance_avoid_invalid_utf8();
      true
    } else {
      false
    }
  }

  /// Look for an invalid utf-8 character right ahead and return if any
  /// INVARIANT: Always call before any other advance()/consume()
  fn try_consume_invalid_utf8(&mut self) -> Option<LexResult> {
    debug_assert!(
      self.text_buffer.len() == 0,
      "Do not call advance()/consume() before try_consume_invalid_utf8()"
    );
    if let Utf8Result::Invalid { len, bytes } = self.peek() {
      Some(LexResult {
        token: self
          .cache
          .borrow_mut()
          .token(SyntaxKind::Error, &bytes[..len]),
        diagnostic: Some(LexDiagnostic::InvalidUtf8 {
          start_offset: self.stream.offset() - len,
          end_offset: self.stream.offset(),
        }),
      })
    } else {
      None
    }
  }

  /// Finalize the current token with no diagnostic.
  fn emit(&mut self, kind: SyntaxKind) -> LexResult {
    let text = std::mem::take(&mut self.text_buffer);
    LexResult {
      token: self.cache.borrow_mut().token(kind, text.as_bytes()),
      diagnostic: None,
    }
  }

  /// Finalize the current token with a diagnostic.
  fn emit_with(&mut self, kind: SyntaxKind, diagnostic: LexDiagnostic) -> LexResult {
    let text = std::mem::take(&mut self.text_buffer);
    LexResult {
      token: self.cache.borrow_mut().token(kind, text.as_bytes()),
      diagnostic: Some(diagnostic),
    }
  }

  /// Whether the stream is exhausted.
  fn is_eof(&mut self) -> bool {
    self.stream.exhausted()
  }
}
