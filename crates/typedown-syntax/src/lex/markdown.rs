use typedown_types::stream::{Utf8Result, Utf8Stream};

use super::ctx::{LexCtx, LexResult, MdInterpContext};
use super::yaml::is_op_char;
use crate::green::syntax_kind::SyntaxKind;
use crate::lex::diagnostic::LexDiagnostic;

// Markdown body lexing
impl<S: Utf8Stream> LexCtx<S> {
  pub(super) fn lex_markdown_body(&mut self) -> LexResult {
    // If inside a formula/string context, dispatch accordingly
    match self.markdown_lex_ctx.interp_stack.last() {
      Some(MdInterpContext::Interpolation) | Some(MdInterpContext::Brace) => {
        return self.lex_markdown_formula();
      }
      Some(MdInterpContext::DqString) | Some(MdInterpContext::SqString) => {
        return self.lex_markdown_resume_string();
      }
      None => {}
    }

    let char = match self.peek() {
      Utf8Result::Char(char) => char,
      _ => panic!(
        "[LexCtx::lex_markdown_body] Expected a valid UTF-8 character but got EOF or invalid bytes."
      ),
    };

    match char {
      /* Newlines */
      '\n' | '\r' => {
        self.advance_avoid_invalid_utf8();
        if char == '\r' {
          self.consume_avoid_invalid_utf8('\n');
        }
        self.emit(SyntaxKind::Newline)
      }

      /* Whitespace */
      _ if char.is_whitespace() => {
        self.advance_avoid_invalid_utf8();
        self.emit(SyntaxKind::Whitespace)
      }

      /* Dollar */
      '$' => self.lex_markdown_dollar(),

      /* Brackets and parens */
      '[' => {
        self.advance_avoid_invalid_utf8();
        self.emit(SyntaxKind::LBracket)
      }
      ']' => {
        self.advance_avoid_invalid_utf8();
        self.emit(SyntaxKind::RBracket)
      }
      '(' => {
        self.advance_avoid_invalid_utf8();
        self.emit(SyntaxKind::LParen)
      }
      ')' => {
        self.advance_avoid_invalid_utf8();
        self.emit(SyntaxKind::RParen)
      }

      /* Code spans */
      '`' => self.lex_markdown_code(),

      /* Numbers */
      '0'..='9' => self.lex_markdown_number(),

      /* Symbols */
      _ if is_md_symbol_char(char) => self.lex_markdown_symbol(),

      /* Everything else is text */
      _ => self.lex_markdown_text(),
    }
  }

  /* Dollar */

  fn lex_markdown_dollar(&mut self) -> LexResult {
    self.advance_avoid_invalid_utf8(); // consume $
    if self.consume_avoid_invalid_utf8('{') {
      // ${ enters formula mode
      self
        .markdown_lex_ctx
        .interp_stack
        .push(MdInterpContext::Interpolation);
      self.emit(SyntaxKind::InterpStart)
    } else {
      // $ enters inline math
      self.lex_inline_math_content()
    }
  }

  /* Text */

  fn lex_markdown_text(&mut self) -> LexResult {
    loop {
      match self.peek() {
        Utf8Result::Char(char)
          if !char.is_whitespace()
            && char != '$'
            && char != '['
            && char != ']'
            && char != '('
            && char != ')'
            && char != '`'
            && !is_md_symbol_char(char)
            && !char.is_ascii_digit() =>
        {
          self.advance_avoid_invalid_utf8();
        }
        _ => break,
      }
    }
    self.emit(SyntaxKind::Ident)
  }

  /* Symbols */

  fn lex_markdown_symbol(&mut self) -> LexResult {
    loop {
      match self.peek() {
        Utf8Result::Char(char) if is_md_symbol_char(char) => {
          self.advance_avoid_invalid_utf8();
        }
        _ => break,
      }
    }
    self.emit(SyntaxKind::MdSymbol)
  }

  /* Code spans */

  fn lex_markdown_code(&mut self) -> LexResult {
    // Count opening backticks
    let mut fence_count = 0;
    while let Utf8Result::Char('`') = self.peek() {
      self.advance_avoid_invalid_utf8();
      fence_count += 1;
    }

    // Check if content starts with a newline (block) or not (inline)
    let is_block = matches!(self.peek(), Utf8Result::Char('\n') | Utf8Result::Char('\r'));

    // Consume content until matching fence count
    loop {
      match self.peek() {
        Utf8Result::Char('`') => {
          let mut count = 0;
          while let Utf8Result::Char('`') = self.peek() {
            self.advance_avoid_invalid_utf8();
            count += 1;
            if count == fence_count {
              break;
            }
          }
          if count == fence_count {
            let kind = if is_block {
              SyntaxKind::CodeBlock
            } else {
              SyntaxKind::InlineCode
            };
            return self.emit(kind);
          }
        }
        Utf8Result::Eof => {
          let start = self.stream.offset() - self.text_buffer.len();
          let end = self.stream.offset();
          return self.emit_with(
            SyntaxKind::Error,
            LexDiagnostic::UnterminatedCodeBlock {
              start_offset: start,
              end_offset: end,
            },
          );
        }
        _ => {
          self.advance_avoid_invalid_utf8();
        }
      }
    }
  }

  /* Numbers */

  fn lex_markdown_number(&mut self) -> LexResult {
    self.lex_number()
  }

  /* Formula mode (inside ${...} in markdown) */

  fn lex_markdown_formula(&mut self) -> LexResult {
    if let Utf8Result::Eof = self.peek() {
      self.markdown_lex_ctx.interp_stack.pop();
      let offset = self.stream.offset();
      return self.emit_with(
        SyntaxKind::Error,
        LexDiagnostic::UnterminatedInterpolation {
          start_offset: offset,
          end_offset: offset,
        },
      );
    }

    let char = match self.peek() {
      Utf8Result::Char(char) => char,
      _ => unreachable!(),
    };

    match char {
      '}' => {
        self.advance_avoid_invalid_utf8();
        match self.markdown_lex_ctx.interp_stack.last() {
          Some(MdInterpContext::Brace) => {
            self.markdown_lex_ctx.interp_stack.pop();
            self.emit(SyntaxKind::RBrace)
          }
          Some(MdInterpContext::Interpolation) => {
            self.markdown_lex_ctx.interp_stack.pop();
            self.emit(SyntaxKind::InterpEnd)
          }
          _ => self.emit(SyntaxKind::RBrace),
        }
      }
      '{' => {
        self.advance_avoid_invalid_utf8();
        self
          .markdown_lex_ctx
          .interp_stack
          .push(MdInterpContext::Brace);
        self.emit(SyntaxKind::LBrace)
      }
      '"' => {
        self.advance_avoid_invalid_utf8();
        self
          .markdown_lex_ctx
          .interp_stack
          .push(MdInterpContext::DqString);
        self.emit(SyntaxKind::DqStrStart)
      }
      '\'' => {
        self.advance_avoid_invalid_utf8();
        self
          .markdown_lex_ctx
          .interp_stack
          .push(MdInterpContext::SqString);
        self.emit(SyntaxKind::SqStrStart)
      }
      '\n' | '\r' => {
        self.advance_avoid_invalid_utf8();
        if char == '\r' {
          self.consume_avoid_invalid_utf8('\n');
        }
        self.emit(SyntaxKind::Newline)
      }
      _ if char.is_whitespace() => {
        self.advance_avoid_invalid_utf8();
        self.emit(SyntaxKind::Whitespace)
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
      ',' => {
        self.advance_avoid_invalid_utf8();
        self.emit(SyntaxKind::Comma)
      }
      ':' => {
        self.advance_avoid_invalid_utf8();
        self.emit(SyntaxKind::Colon)
      }
      '0'..='9' => self.lex_number(),
      _ if char.is_alphabetic() || char == '_' => {
        loop {
          match self.peek() {
            Utf8Result::Char(char) if char.is_alphanumeric() || char == '_' => {
              self.advance_avoid_invalid_utf8();
            }
            _ => break,
          }
        }
        self.emit(SyntaxKind::Ident)
      }
      _ if is_op_char(char) => {
        self.consume_op_chars();
        self.emit(SyntaxKind::YamlOp)
      }
      _ => {
        self.advance_avoid_invalid_utf8();
        self.emit(SyntaxKind::Error)
      }
    }
  }

  fn lex_markdown_resume_string(&mut self) -> LexResult {
    match self.markdown_lex_ctx.interp_stack.last() {
      Some(MdInterpContext::DqString) => {
        self.lex_markdown_string_content('"', SyntaxKind::DqStrContent, SyntaxKind::DqStrEnd)
      }
      Some(MdInterpContext::SqString) => {
        self.lex_markdown_string_content('\'', SyntaxKind::SqStrContent, SyntaxKind::SqStrEnd)
      }
      _ => panic!(
        "[LexCtx::lex_markdown_resume_string] Called without a string context on the interp stack"
      ),
    }
  }

  fn lex_markdown_string_content(
    &mut self,
    closing: char,
    content_kind: SyntaxKind,
    end_kind: SyntaxKind,
  ) -> LexResult {
    loop {
      match self.peek() {
        Utf8Result::Char(char) if char == closing => {
          if self.text_buffer.is_empty() {
            self.advance_avoid_invalid_utf8();
            self.markdown_lex_ctx.interp_stack.pop();
            return self.emit(end_kind);
          } else {
            let content = self.emit(content_kind);
            self.advance_avoid_invalid_utf8();
            self.markdown_lex_ctx.interp_stack.pop();
            let end = self.emit(end_kind);
            self.pending_tokens.push(end);
            return content;
          }
        }
        Utf8Result::Char('$') => {
          self.advance_avoid_invalid_utf8();
          match self.peek() {
            Utf8Result::Char('{') => {
              self.advance_avoid_invalid_utf8();
              let buf_len = self.text_buffer.len();
              let string_text: String = self.text_buffer.drain(..buf_len - 2).collect();
              self.text_buffer.clear();

              self
                .markdown_lex_ctx
                .interp_stack
                .push(MdInterpContext::Interpolation);

              let interp_start = LexResult {
                token: self
                  .cache
                  .borrow_mut()
                  .token(SyntaxKind::InterpStart, "${".as_bytes()),
                diagnostic: None,
              };

              if !string_text.is_empty() {
                let content = LexResult {
                  token: self
                    .cache
                    .borrow_mut()
                    .token(content_kind, string_text.as_bytes()),
                  diagnostic: None,
                };
                self.pending_tokens.push(interp_start);
                return content;
              } else {
                return interp_start;
              }
            }
            _ => {
              // Single $ inside string: inline math
              let buf_len = self.text_buffer.len();
              let string_text: String = self.text_buffer.drain(..buf_len - 1).collect();
              self.text_buffer.clear();

              let math_token = self.lex_inline_math_content();

              if !string_text.is_empty() {
                let content = LexResult {
                  token: self
                    .cache
                    .borrow_mut()
                    .token(content_kind, string_text.as_bytes()),
                  diagnostic: None,
                };
                self.pending_tokens.push(math_token);
                return content;
              } else {
                return math_token;
              }
            }
          }
        }
        Utf8Result::Char('\\') => {
          self.advance_avoid_invalid_utf8();
          match self.peek() {
            Utf8Result::Char('\n') | Utf8Result::Char('\r') | Utf8Result::Eof => {
              let start = self.stream.offset() - self.text_buffer.len();
              let end = self.stream.offset();
              self.markdown_lex_ctx.interp_stack.pop();
              return self.emit_with(
                SyntaxKind::Error,
                LexDiagnostic::UnterminatedString {
                  start_offset: start,
                  end_offset: end,
                },
              );
            }
            _ => {
              self.advance_avoid_invalid_utf8();
            }
          }
        }
        Utf8Result::Char('\n') | Utf8Result::Char('\r') | Utf8Result::Eof => {
          let start = self.stream.offset() - self.text_buffer.len();
          let end = self.stream.offset();
          self.markdown_lex_ctx.interp_stack.pop();
          return self.emit_with(
            SyntaxKind::Error,
            LexDiagnostic::UnterminatedString {
              start_offset: start,
              end_offset: end,
            },
          );
        }
        _ => {
          self.advance_avoid_invalid_utf8();
        }
      }
    }
  }
}

fn is_md_symbol_char(char: char) -> bool {
  matches!(
    char,
    '#'
      | '!'
      | '*'
      | '~'
      | '^'
      | '-'
      | '>'
      | '<'
      | '|'
      | '@'
      | ':'
      | '\\'
      | '/'
      | '='
      | '+'
      | '&'
      | '%'
  )
}
