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

  // State for YAML lexing
  yaml_lex_ctx: YamlLexCtx,
  // State for Markdown lexing
  markdown_lex_ctx: MarkdownLexCtx,
}

/* YAML */
struct YamlLexCtx {
  // Whether we're just after a newline (linux), CRLF (Windows), carriage return (Mac)
  at_line_start: bool,

  // Indent stack for YAML
  // In block style, YAML is indentation-sensitive
  // We keep track of the previous indentations
  indent_stack: Vec<usize>,

  // We allow nested interpolations
  // We need to distinguish between nested strings, interpolations, etc.
  interp_stack: Vec<YamlInterpContext>,

  // The indent character established by the first indented line (None = not yet determined)
  indent_char: Option<char>,

  // Pending dedent count to emit before the next real token.
  pending_dedents: usize,
}

enum YamlInterpContext {
  SqString,
  DqString,
}

struct MarkdownLexCtx {}

impl<S: Utf8Stream> LexCtx<S> {
  pub fn new(stream: S, cache: Rc<RefCell<Cache>>) -> Self {
    Self {
      stream,
      cache,
      mode: LexMode::YamlFrontmatter,
      text_buffer: String::from(""),
      yaml_lex_ctx: YamlLexCtx {
        at_line_start: true,
        indent_stack: vec![0],
        interp_stack: vec![],
        indent_char: None,
        pending_dedents: 0,
      },
      markdown_lex_ctx: MarkdownLexCtx {},
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
        LexMode::YamlFrontmatter => self.lex_yaml_frontmatter(),
        LexMode::MarkdownBody => self.lex_markdown_body(),
      }
    }
  }
}

// YAML frontmatter lexing
impl<S: Utf8Stream> LexCtx<S> {
  fn lex_yaml_frontmatter(&mut self) -> LexResult {
    // Emit pending dedents one at a time
    if self.yaml_lex_ctx.pending_dedents > 0 {
      self.yaml_lex_ctx.pending_dedents -= 1;
      return self.emit(SyntaxKind::Dedent);
    }

    // At line start, handle indentation
    if self.yaml_lex_ctx.at_line_start {
      if let Some(result) = self.lex_yaml_indent() {
        return result;
      }
    }

    let char = match self.peek() {
      Utf8Result::Char(char) => char,
      _ => panic!("[LexCtx::lex_yaml_frontmatter] Expected a valid UTF-8 character but got EOF or invalid bytes. This should have been handled by try_consume_invalid_utf8 or is_eof before reaching this point."),
    };

    match char {
      /* Newlines */
      '\n' | '\r' => {
        self.advance_avoid_invalid_utf8();
        if char == '\r' {
          self.consume_avoid_invalid_utf8('\n');
        }
        self.yaml_lex_ctx.at_line_start = true;
        self.emit(SyntaxKind::Newline)
      }

      /* Whitespace */
      _ if char.is_whitespace() => self.lex_yaml_whitespaces(),

      /* Comments */
      '#' => self.lex_yaml_comment(),

      /* Punctuation and delimiters */
      ':' => {
        self.advance_avoid_invalid_utf8();
        self.emit(SyntaxKind::Colon)
      }
      '!' => {
        self.advance_avoid_invalid_utf8();
        self.emit(SyntaxKind::Bang)
      }
      '(' => {
        self.advance_avoid_invalid_utf8();
        self.emit(SyntaxKind::LParen)
      }
      ')' => {
        self.advance_avoid_invalid_utf8();
        self.emit(SyntaxKind::RParen)
      }
      '[' => {
        self.advance_avoid_invalid_utf8();
        self.emit(SyntaxKind::LBracket)
      }
      ']' => {
        self.advance_avoid_invalid_utf8();
        self.emit(SyntaxKind::RBracket)
      }
      '{' => {
        self.advance_avoid_invalid_utf8();
        self.emit(SyntaxKind::LBrace)
      }
      '}' => {
        self.advance_avoid_invalid_utf8();
        self.emit(SyntaxKind::RBrace)
      }
      ',' => {
        self.advance_avoid_invalid_utf8();
        self.emit(SyntaxKind::Comma)
      }

      /* Strings */
      '"' => self.lex_yaml_dq_string(),
      '\'' => self.lex_yaml_sq_string(),

      /* Dollar and interpolation */
      '$' => self.lex_yaml_dollar(),

      /* Numbers */
      '0'..='9' => self.lex_yaml_number(),

      /* Identifiers */
      _ if char.is_alphabetic() || char == '_' => self.lex_yaml_ident(),

      /* Operators */
      '-' | '+' | '*' | '/' | '\\' | '.' | '~' | '^' | '|' | '>' | '<' | '=' | '%' | '&' | '@' => {
        self.lex_yaml_op()
      }

      /* Error fallback */
      _ => {
        self.advance_avoid_invalid_utf8();
        self.emit(SyntaxKind::Error)
      }
    }
  }

  /* Indentation */

  fn current_indent(&self) -> usize {
    *self.yaml_lex_ctx.indent_stack.last().unwrap_or(&0)
  }

  fn lex_yaml_indent(&mut self) -> Option<LexResult> {
    self.yaml_lex_ctx.at_line_start = false;

    // Count leading whitespace and track tab/space usage
    let start = self.stream.offset();
    let mut indent = 0;
    let mut saw_space = false;
    let mut saw_tab = false;
    loop {
      match self.peek() {
        Utf8Result::Char(char) if char.is_whitespace() && char != '\n' && char != '\r' => {
          if char == '\t' {
            saw_tab = true;
          } else {
            saw_space = true;
          }
          indent += 1;
          self.advance_avoid_invalid_utf8();
        }
        _ => break,
      }
    }

    // Detect mixed or inconsistent indentation
    let diagnostic = if indent > 0 {
      if saw_space && saw_tab {
        // Mixed tabs and spaces on the same line
        Some(LexDiagnostic::MixedIndentation {
          start_offset: start,
          end_offset: self.stream.offset(),
        })
      } else {
        let char = if saw_tab { '\t' } else { ' ' };
        match self.yaml_lex_ctx.indent_char {
          None => {
            // Establish the indent character
            self.yaml_lex_ctx.indent_char = Some(char);
            None
          }
          Some(established) if established != char => {
            // Different indent character than established
            Some(LexDiagnostic::InconsistentIndentation {
              expected: established,
              encountered: char,
              start_offset: start,
              end_offset: self.stream.offset(),
            })
          }
          _ => None,
        }
      }
    } else {
      None
    };

    let current = self.current_indent();

    if indent > current {
      // Increased indentation
      self.yaml_lex_ctx.indent_stack.push(indent);
      self.text_buffer.clear();
      return Some(match diagnostic {
        Some(diag) => self.emit_with(SyntaxKind::Indent, diag),
        None => self.emit(SyntaxKind::Indent),
      });
    } else if indent < current {
      // Decreased indentation: pop levels until we find one <= indent
      let mut dedents = 0;
      while let Some(&top) = self.yaml_lex_ctx.indent_stack.last() {
        if top > indent {
          self.yaml_lex_ctx.indent_stack.pop();
          dedents += 1;
        } else {
          break;
        }
      }

      // If indent doesn't match an existing level exactly, emit an error
      let diagnostic = if indent != self.current_indent() {
        Some(diagnostic.unwrap_or(LexDiagnostic::UnmatchedDedent {
          indent,
          start_offset: start,
          end_offset: self.stream.offset(),
        }))
      } else {
        diagnostic
      };

      if dedents > 0 {
        // Queue remaining dedents, emit the first one now
        self.yaml_lex_ctx.pending_dedents = dedents - 1;
        self.text_buffer.clear();
        return Some(match diagnostic {
          Some(diag) => self.emit_with(SyntaxKind::Dedent, diag),
          None => self.emit(SyntaxKind::Dedent),
        });
      }
    }

    // Same indent level: emit whitespace if any was consumed
    if !self.text_buffer.is_empty() {
      return Some(self.emit(SyntaxKind::Whitespace));
    }

    None
  }

  /* Whitespace */

  fn lex_yaml_whitespaces(&mut self) -> LexResult {
    self.advance_avoid_invalid_utf8();
    self.emit(SyntaxKind::Whitespace)
  }

  /* Comments */

  fn lex_yaml_comment(&mut self) -> LexResult {
    self.advance_avoid_invalid_utf8(); // consume #
    loop {
      match self.peek() {
        Utf8Result::Char(char) if char != '\n' && char != '\r' => {
          self.advance_avoid_invalid_utf8();
        }
        _ => break,
      }
    }
    self.emit(SyntaxKind::Comment)
  }

  /* Operators */

  fn lex_yaml_op(&mut self) -> LexResult {
    self.consume_op_chars();
    self.emit(SyntaxKind::YamlOp)
  }

  fn consume_op_chars(&mut self) {
    loop {
      match self.peek() {
        Utf8Result::Char(char) if is_op_char(char) => {
          self.advance_avoid_invalid_utf8();
        }
        _ => break,
      }
    }
  }

  /* Strings */

  fn lex_yaml_dq_string(&mut self) -> LexResult { todo!() }
  fn lex_yaml_sq_string(&mut self) -> LexResult { todo!() }

  /* Dollar and interpolation */

  fn lex_yaml_dollar(&mut self) -> LexResult { todo!() }

  /* Numbers */

  fn lex_yaml_number(&mut self) -> LexResult { todo!() }

  /* Identifiers */

  fn lex_yaml_ident(&mut self) -> LexResult { todo!() }
}

fn is_op_char(char: char) -> bool {
  matches!(
    char,
    '+' | '-' | '*' | '/' | '\\' | '.' | '~' | '^' | '|' | '>' | '<' | '=' | '%' | '&' | '@'
  )
}

// Markdown body lexing
impl<S: Utf8Stream> LexCtx<S> {
  fn lex_markdown_body(&mut self) -> LexResult {
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
