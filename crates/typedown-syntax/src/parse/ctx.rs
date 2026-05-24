//! A recursive descent parser

use std::{cell::RefCell, rc::Rc};

use typedown_types::{diagnostic::Diagnostic, stream::Utf8Stream};

use crate::{
  green::{GreenNode, SyntaxToken, cache::Cache, syntax_kind::SyntaxKind},
  lex::ctx::{LexCtx, LexMode, LexResult},
  parse::peekable_lex_ctx::PeekableLexCtx,
};

pub struct ParseCtx<S: Utf8Stream> {
  pub(super) cache: Rc<RefCell<Cache>>,
  pub(super) lex_ctx: PeekableLexCtx<S>,
  pub(super) diagnostics: Vec<Diagnostic>,
  ast: Option<GreenNode>,
}

pub struct ParseResult<'a> {
  ast: GreenNode,
  diagnostics: &'a [Diagnostic],
}

impl<S: Utf8Stream> ParseCtx<S> {
  pub fn new(stream: S, cache: Rc<RefCell<Cache>>) -> ParseCtx<S> {
    Self {
      cache: cache.clone(),
      lex_ctx: PeekableLexCtx::new(LexCtx::new(stream, cache)),
      diagnostics: Vec::new(),
      ast: None,
    }
  }

  pub fn parse<'a>(&'a mut self) -> ParseResult<'a> {
    if let Some(ref ast) = self.ast {
      ParseResult {
        ast: ast.clone(),
        diagnostics: &self.diagnostics,
      }
    } else {
      let root = self.parse_source_file();
      self.ast = Some(root.clone());
      ParseResult {
        ast: root,
        diagnostics: &self.diagnostics,
      }
    }
  }

  pub(super) fn parse_source_file(&mut self) -> GreenNode {
    let yaml_frontmatter = self.parse_yaml_frontmatter();
    self.lex_ctx.set_mode(LexMode::MarkdownBody);
    let markdown_body = self.parse_markdown_body();
    self.emit(
      SyntaxKind::SourceFile,
      &vec![yaml_frontmatter, markdown_body],
    )
  }
}

impl<S: Utf8Stream> ParseCtx<S> {
  /// Consume the next non-skipped YAML token, pushing skipped trivia and result into children.
  pub(super) fn advance_yaml(&mut self, children: &mut Vec<GreenNode>, skip: u16) -> LexResult {
    loop {
      let mut result = self.lex_ctx.lex();
      if let Some(diagnostic) = result.diagnostic.take() {
        self.diagnostics.push(diagnostic);
      }
      if self.lex_ctx.should_skip(result.token.kind(), skip) {
        children.push(GreenNode::from_token(result.token));
      } else {
        children.push(GreenNode::from_token(result.token.clone()));
        return result;
      }
    }
  }

  /// Consume the next non-skipped Markdown token, pushing skipped trivia and result into children.
  pub(super) fn advance_md(&mut self, children: &mut Vec<GreenNode>, skip: u16) -> LexResult {
    loop {
      let mut result = self.lex_ctx.lex();
      if let Some(diagnostic) = result.diagnostic.take() {
        self.diagnostics.push(diagnostic);
      }
      if self.lex_ctx.should_skip(result.token.kind(), skip) {
        children.push(GreenNode::from_token(result.token));
      } else {
        children.push(GreenNode::from_token(result.token.clone()));
        return result;
      }
    }
  }

  pub fn advance(&mut self, children: &mut Vec<GreenNode>, skip: u16, mode: LexMode) -> LexResult {
    debug_assert!(
      *self.lex_ctx.mode() == mode,
      "[PeekableLexCtx::advance] Lex mode must be the same as the `mode` argument"
    );
    match mode {
      LexMode::YamlFrontmatter => self.advance_yaml(children, skip),
      LexMode::MarkdownBody => self.advance_md(children, skip),
    }
  }

  /// Like advance_yaml(), but expects the token to match `expected`.
  pub(super) fn consume_yaml(
    &mut self,
    children: &mut Vec<GreenNode>,
    skip: u16,
    expected: SyntaxKind,
    diagnostic: Diagnostic,
  ) -> bool {
    let result = self.advance_yaml(children, skip);
    if result.token.kind() != expected {
      let bad_token = children.pop().unwrap();
      children.push(self.emit(SyntaxKind::Error, &[bad_token]));
      self.diagnostics.push(diagnostic);
      false
    } else {
      true
    }
  }

  /// Like advance_md(), but expects the token to match `expected`.
  pub(super) fn consume_md(
    &mut self,
    children: &mut Vec<GreenNode>,
    skip: u16,
    expected: SyntaxKind,
    diagnostic: Diagnostic,
  ) -> bool {
    let result = self.advance_md(children, skip);
    if result.token.kind() != expected {
      let bad_token = children.pop().unwrap();
      children.push(self.emit(SyntaxKind::Error, &[bad_token]));
      self.diagnostics.push(diagnostic);
      false
    } else {
      true
    }
  }

  pub fn consume(
    &mut self,
    children: &mut Vec<GreenNode>,
    skip: u16,
    mode: LexMode,
    expected: SyntaxKind,
    diagnostic: Diagnostic,
  ) -> bool {
    debug_assert!(
      *self.lex_ctx.mode() == mode,
      "[PeekableLexCtx::consume] Lex mode must be the same as the `mode` argument"
    );
    match mode {
      LexMode::YamlFrontmatter => self.consume_yaml(children, skip, expected, diagnostic),
      LexMode::MarkdownBody => self.consume_md(children, skip, expected, diagnostic),
    }
  }

  /// Like advance_yaml(), but expects the token to satisfy a predicate.
  pub(super) fn consume_yaml_if(
    &mut self,
    children: &mut Vec<GreenNode>,
    skip: u16,
    predicate: impl Fn(&SyntaxToken) -> bool,
    diagnostic: Diagnostic,
  ) -> bool {
    let result = self.advance_yaml(children, skip);
    if !predicate(&result.token) {
      let bad_token = children.pop().unwrap();
      children.push(self.emit(SyntaxKind::Error, &[bad_token]));
      self.diagnostics.push(diagnostic);
      false
    } else {
      true
    }
  }

  /// Like advance_md(), but expects the token to satisfy a predicate.
  pub(super) fn consume_md_if(
    &mut self,
    children: &mut Vec<GreenNode>,
    skip: u16,
    predicate: impl Fn(&SyntaxToken) -> bool,
    diagnostic: Diagnostic,
  ) -> bool {
    let result = self.advance_md(children, skip);
    if !predicate(&result.token) {
      let bad_token = children.pop().unwrap();
      children.push(self.emit(SyntaxKind::Error, &[bad_token]));
      self.diagnostics.push(diagnostic);
      false
    } else {
      true
    }
  }

  pub fn consume_if(
    &mut self,
    children: &mut Vec<GreenNode>,
    skip: u16,
    mode: LexMode,
    predicate: impl Fn(&SyntaxToken) -> bool,
    diagnostic: Diagnostic,
  ) -> bool {
    debug_assert!(
      *self.lex_ctx.mode() == mode,
      "[PeekableLexCtx::consume_if] Lex mode must be the same as the `mode` argument"
    );
    match mode {
      LexMode::YamlFrontmatter => self.consume_yaml_if(children, skip, predicate, diagnostic),
      LexMode::MarkdownBody => self.consume_md_if(children, skip, predicate, diagnostic),
    }
  }

  /// Current byte offset in the source stream.
  pub(super) fn offset(&self) -> usize {
    self.lex_ctx.offset()
  }

  /// Emit a GreenNode
  pub(super) fn emit(&mut self, kind: SyntaxKind, children: &[GreenNode]) -> GreenNode {
    GreenNode::from_node(self.cache.borrow_mut().node(kind, children))
  }
}
