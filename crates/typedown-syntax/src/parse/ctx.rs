//! A recursive descent parser

use std::{cell::RefCell, rc::Rc};

use typedown_types::{diagnostic::Diagnostic, stream::Utf8Stream};

use crate::{
  green::{GreenNode, SyntaxToken, cache::Cache, syntax_kind::SyntaxKind},
  lex::ctx::{LexCtx, LexMode, LexResult},
};

pub struct ParseCtx<S: Utf8Stream> {
  pub(super) cache: Rc<RefCell<Cache>>,
  pub(super) lex_ctx: LexCtx<S>,
  ast: Option<(GreenNode, Vec<Diagnostic>)>,
}

pub struct ParseResult<'a> {
  ast: GreenNode,
  diagnostics: &'a [Diagnostic],
}

impl<S: Utf8Stream> ParseCtx<S> {
  pub fn new(stream: S, cache: Rc<RefCell<Cache>>) -> ParseCtx<S> {
    Self {
      cache: cache.clone(),
      lex_ctx: LexCtx::new(stream, cache),
      ast: None,
    }
  }

  pub fn parse<'a>(&'a mut self) -> ParseResult<'a> {
    if let Some((ref ast, ref diagnostics)) = self.ast {
      ParseResult {
        ast: ast.clone(),
        diagnostics,
      }
    } else {
      self.parse_source_file();
      let (ast, diagnostics) = self
        .ast
        .as_ref()
        .expect("ParseCtx::ast should be set after calling parse_source_file()");
      ParseResult {
        ast: ast.clone(),
        diagnostics,
      }
    }
  }

  fn parse_source_file(&mut self) -> GreenNode {
    let yaml_frontmatter = self.parse_yaml_frontmatter();
    self.lex_ctx.set_mode(LexMode::MarkdownBody);
    let markdown_body = self.parse_markdown_body();
    self.emit(
      SyntaxKind::SourceFile,
      &vec![yaml_frontmatter, markdown_body],
    )
  }
}

/// We do not support peek here, because lexing is irreversible
impl<S: Utf8Stream> ParseCtx<S> {
  /// Consume the next token, push it into children, and return it.
  pub(super) fn advance(&mut self, children: &mut Vec<GreenNode>) -> LexResult {
    let result = self.lex_ctx.lex();
    children.push(GreenNode::from_token(result.token.clone()));
    result
  }

  /// Consume tokens, skipping whitespace. Skipped tokens are pushed into children.
  /// Returns the first non-whitespace token.
  pub(super) fn advance_skip_ws(&mut self, children: &mut Vec<GreenNode>) -> LexResult {
    loop {
      let result = self.lex_ctx.lex();
      if result.token.kind() == SyntaxKind::Whitespace {
        children.push(GreenNode::from_token(result.token));
      } else {
        children.push(GreenNode::from_token(result.token.clone()));
        return result;
      }
    }
  }

  /// Consume tokens, skipping whitespace and comments. Skipped tokens are pushed into children.
  /// Returns the first non-whitespace, non-comment token.
  pub(super) fn advance_skip_wc(&mut self, children: &mut Vec<GreenNode>) -> LexResult {
    loop {
      let result = self.lex_ctx.lex();
      match result.token.kind() {
        SyntaxKind::Whitespace | SyntaxKind::YamlComment => {
          children.push(GreenNode::from_token(result.token));
        }
        _ => {
          children.push(GreenNode::from_token(result.token.clone()));
          return result;
        }
      }
    }
  }

  /// Consume tokens, skipping whitespace, comments, and newlines. Skipped tokens are pushed into children.
  /// Returns the first non-trivia token.
  pub(super) fn advance_skip_wcn(&mut self, children: &mut Vec<GreenNode>) -> LexResult {
    loop {
      let result = self.lex_ctx.lex();
      match result.token.kind() {
        SyntaxKind::Whitespace | SyntaxKind::YamlComment | SyntaxKind::Newline => {
          children.push(GreenNode::from_token(result.token));
        }
        _ => {
          children.push(GreenNode::from_token(result.token.clone()));
          return result;
        }
      }
    }
  }

  /// Emit a GreenNode
  pub(super) fn emit(&mut self, kind: SyntaxKind, children: &[GreenNode]) -> GreenNode {
    GreenNode::from_node(self.cache.borrow_mut().node(kind, children))
  }
}
