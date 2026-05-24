//! A recursive descent parser

pub(in crate::parse) mod expr_ctx;
pub(in crate::parse) mod peekable_lex_ctx;

use std::{cell::RefCell, rc::Rc};

use typedown_types::{diagnostic::Diagnostic, stream::Utf8Stream, syntax_kind::SyntaxKind};

use crate::{
  green::{GreenNode, SyntaxToken, cache::Cache},
  lex::ctx::{LexCtx, LexMode},
};
use expr_ctx::ExprCtxStack;
use peekable_lex_ctx::AugmentedLexResult;
use peekable_lex_ctx::PeekableLexCtx;

pub struct ParseCtx<S: Utf8Stream> {
  pub(in crate::parse) cache: Rc<RefCell<Cache>>,
  pub(in crate::parse) lex_ctx: PeekableLexCtx<S>,
  pub(in crate::parse) diagnostics: Vec<Diagnostic>,
  pub(in crate::parse) expr_ctx_stack: ExprCtxStack,
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
      expr_ctx_stack: ExprCtxStack::new(),
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

  pub(in crate::parse) fn parse_source_file(&mut self) -> GreenNode {
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
  pub(in crate::parse) fn advance_yaml(
    &mut self,
    children: &mut Vec<GreenNode>,
    skip: u16,
  ) -> AugmentedLexResult {
    loop {
      let mut result = self.lex_ctx.lex();
      if let Some(diagnostic) = result.diagnostic.take() {
        self.diagnostics.push(diagnostic);
      }
      if self.lex_ctx.should_skip(result.token.kind(), skip) {
        children.push(GreenNode::from_token(result.token));
      } else {
        let indent_depth = self.lex_ctx.yaml_indent_depth();
        children.push(GreenNode::from_token(result.token.clone()));
        return AugmentedLexResult::new(result, indent_depth);
      }
    }
  }

  /// Consume the next non-skipped Markdown token, pushing skipped trivia and result into children.
  pub(in crate::parse) fn advance_md(
    &mut self,
    children: &mut Vec<GreenNode>,
    skip: u16,
  ) -> AugmentedLexResult {
    loop {
      let mut result = self.lex_ctx.lex();
      if let Some(diagnostic) = result.diagnostic.take() {
        self.diagnostics.push(diagnostic);
      }
      if self.lex_ctx.should_skip(result.token.kind(), skip) {
        children.push(GreenNode::from_token(result.token));
      } else {
        let indent_depth = self.lex_ctx.md_indent_depth();
        children.push(GreenNode::from_token(result.token.clone()));
        return AugmentedLexResult::new(result, indent_depth);
      }
    }
  }

  pub fn advance(
    &mut self,
    children: &mut Vec<GreenNode>,
    skip: u16,
    mode: LexMode,
  ) -> AugmentedLexResult {
    debug_assert!(
      self.lex_ctx.mode() == mode,
      "[PeekableLexCtx::advance] Lex mode must be the same as the `mode` argument"
    );
    match mode {
      LexMode::YamlFrontmatter => self.advance_yaml(children, skip),
      LexMode::MarkdownBody => self.advance_md(children, skip),
    }
  }

  /// Like advance_yaml(), but expects the token to match `expected`.
  pub(in crate::parse) fn consume_yaml(
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
  pub(in crate::parse) fn consume_md(
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
      self.lex_ctx.mode() == mode,
      "[PeekableLexCtx::consume] Lex mode must be the same as the `mode` argument"
    );
    match mode {
      LexMode::YamlFrontmatter => self.consume_yaml(children, skip, expected, diagnostic),
      LexMode::MarkdownBody => self.consume_md(children, skip, expected, diagnostic),
    }
  }

  /// Like advance_yaml(), but expects the token to satisfy a predicate.
  pub(in crate::parse) fn consume_yaml_if(
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
  pub(in crate::parse) fn consume_md_if(
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
      self.lex_ctx.mode() == mode,
      "[PeekableLexCtx::consume_if] Lex mode must be the same as the `mode` argument"
    );
    match mode {
      LexMode::YamlFrontmatter => self.consume_yaml_if(children, skip, predicate, diagnostic),
      LexMode::MarkdownBody => self.consume_md_if(children, skip, predicate, diagnostic),
    }
  }

  /// Current byte offset in the source stream.
  pub(in crate::parse) fn offset(&self) -> usize {
    self.lex_ctx.offset()
  }

  /// Emit a GreenNode
  pub(in crate::parse) fn emit(&mut self, kind: SyntaxKind, children: &[GreenNode]) -> GreenNode {
    GreenNode::from_node(self.cache.borrow_mut().node(kind, children))
  }
}
