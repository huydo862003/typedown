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
use crate::green::syntax_kind::{self, SyntaxKind};
use crate::green::token::Token;
use crate::lex::diagnostic::{self, LexDiagnostic};

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
    } else if let Utf8Result::Invalid {
      start_offset,
      end_offset,
    } = self.peek()
    {
      self.advance();
      self.emit_with(
        SyntaxKind::Error,
        LexDiagnostic::InvalidUtf8 {
          start_offset,
          end_offset,
        },
      )
    } else {
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
  fn advance(&mut self) -> Utf8Result {
    self.stream.advance()
  }

  /// Consume the next character if it matches `expected`.
  fn consume(&mut self, expected: char) -> bool {
    if let Utf8Result::Char(encountered) = self.peek()
      && encountered == expected
    {
      self.advance();
      true
    } else {
      false
    }
  }

  /// Consume characters while the predicate holds.
  fn consume_while(&mut self, predicate: impl Fn(char) -> bool) {
    while let Utf8Result::Char(encountered) = self.peek() {
      if predicate(encountered) {
        self.advance();
      } else {
        break;
      }
    }
  }

  /// Finalize the current token with no diagnostic.
  fn emit(&mut self, kind: SyntaxKind) -> LexResult {
    let text = std::mem::take(&mut self.text_buffer);
    LexResult {
      token: self.cache.borrow_mut().token(kind, &text),
      diagnostic: None,
    }
  }

  /// Finalize the current token with a diagnostic.
  fn emit_with(&mut self, kind: SyntaxKind, diagnostic: LexDiagnostic) -> LexResult {
    let text = std::mem::take(&mut self.text_buffer);
    LexResult {
      token: self.cache.borrow_mut().token(kind, &text),
      diagnostic: Some(diagnostic),
    }
  }

  /// Whether the stream is exhausted.
  fn is_eof(&mut self) -> bool {
    self.stream.exhausted()
  }
}
