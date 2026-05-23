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

  // Start offset of the to-be-produced token
  start_offset: usize,
  // Current (exclusive end) offset of the to-be-produced token
  end_offset: usize,
  // Text buffer to accumulate the read utf-8
  text_buffer: String,
}

impl<S: Utf8Stream> LexCtx<S> {
  pub fn new(stream: S, cache: Rc<RefCell<Cache>>) -> Self {
    Self {
      stream,
      cache,
      start_offset: 0,
      end_offset: 0,
      mode: LexMode::YamlFrontmatter,
      text_buffer: String::from(""),
    }
  }

  /// Switch the lexing mode.
  pub fn set_mode(&mut self, mode: LexMode) {
    self.mode = mode;
  }

  /// Lex the next token based on the current mode.
  pub fn lex(&mut self) -> Option<LexResult> {
    if self.is_eof() {
      return None;
    }
    let result = match self.mode {
      LexMode::YamlFrontmatter => self.lex_frontmatter(),
      LexMode::MarkdownBody => self.lex_body(),
    };
    Some(result)
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
  fn peek(&mut self) -> Option<char> {
    match self.stream.peek() {
      Utf8Result::Char(ch) => Some(ch),
      _ => None,
    }
  }

  /// Consume the next character, appending it to the current token text.
  fn advance(&mut self) -> Option<char> {
    match self.stream.advance() {
      Utf8Result::Char(ch) => {
        self.end_offset += ch.len_utf8();
        Some(ch)
      }
      _ => None,
    }
  }

  /// Consume the next character if it matches `expected`.
  fn consume(&mut self, expected: char) -> bool {
    if self.peek() == Some(expected) {
      self.advance();
      true
    } else {
      false
    }
  }

  /// Consume characters while the predicate holds.
  fn consume_while(&mut self, predicate: impl Fn(char) -> bool) {
    while let Some(ch) = self.peek() {
      if predicate(ch) {
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
