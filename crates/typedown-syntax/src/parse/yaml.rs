//! YAML frontmatter parsing

use typedown_types::{diagnostic::Diagnostic, stream::Utf8Stream};

use super::constants::*;
use super::ctx::ParseCtx;
use super::peekable_lex_ctx::PeekYamlResult;
use crate::green::{GreenNode, syntax_kind::SyntaxKind};
use crate::lex::ctx::LexMode;

// Top-level YAML frontmatter parsing
impl<S: Utf8Stream> ParseCtx<S> {
  /* Top-level YAML frontmatter */

  pub(super) fn parse_yaml_frontmatter(&mut self) -> GreenNode {
    debug_assert!(
      *self.lex_ctx.mode() == LexMode::YamlFrontmatter,
      "[ParseCtx::parse_yaml_frontmatter] Lex mode must be YamlFrontmatter"
    );

    let mut children = vec![];

    // Consume opening ---
    let ok = self.consume_yaml_if(
      &mut children,
      SKIP_NONE,
      |token| token.kind() == SyntaxKind::YamlOp && token.text().collect::<String>() == "---",
      Diagnostic::MissingFrontmatterMarker {
        offset: self.offset(),
      },
    );
    if !ok {
      self.synchronize_to_triple_dash(&mut children);
    }

    // Expect newline after opening ---
    self.consume_yaml(
      &mut children,
      SKIP_TRAILING_WS | SKIP_COMMENT,
      SyntaxKind::Newline,
      Diagnostic::UnexpectedTokensOnFrontmatterMarkerLine {
        start_offset: self.offset(),
        end_offset: self.offset(),
      },
    );

    // Parse body
    self.parse_yaml_body(&mut children);

    // Consume closing ---
    // Require the indentation to be 0

    let start_offset = self.offset();
    self.consume_yaml_if(
      &mut children,
      SKIP_NEWLINE | SKIP_DEDENT,
      |token| token.kind() == SyntaxKind::YamlOp && token.text().collect::<String>() == "---",
      Diagnostic::MissingFrontmatterMarker {
        offset: start_offset,
      },
    );
    let end_offset = self.offset();

    if self.lex_ctx.indent_depth() != 0 {
      self
        .diagnostics
        .push(Diagnostic::UnexpectedTokensOnFrontmatterMarkerLine {
          start_offset,
          end_offset,
        });
    }

    // Expect newline after closing ---
    self.consume_yaml(
      &mut children,
      SKIP_TRAILING_WS | SKIP_COMMENT,
      SyntaxKind::Newline,
      Diagnostic::UnexpectedTokensOnFrontmatterMarkerLine {
        start_offset: self.offset(),
        end_offset: self.offset(),
      },
    );

    self.emit(SyntaxKind::Frontmatter, &children)
  }

  /// Error recovery: skip tokens until `---` or EOF is found.
  fn synchronize_to_triple_dash(&mut self, children: &mut Vec<GreenNode>) {
    let mut error_children = vec![];

    loop {
      let result = self.lex_ctx.lex();
      let kind = result.token.kind();

      let is_target = (kind == SyntaxKind::YamlOp
        && result.token.text().collect::<String>() == "---")
        || kind == SyntaxKind::Eof;

      if is_target {
        if !error_children.is_empty() {
          children.push(self.emit(SyntaxKind::Error, &error_children));
        }
        children.push(GreenNode::from_token(result.token));
        return;
      }

      error_children.push(GreenNode::from_token(result.token));
    }
  }

  /* YAML frontmatter body */
  pub(super) fn parse_yaml_body(&mut self, children: &mut Vec<GreenNode>) {
    if !self.should_end_yaml_frontmatter() {
      let mapping = self.parse_yaml_block_mapping();
      children.push(mapping);
    }
  }
}

/* YAML mapping */
impl<S: Utf8Stream> ParseCtx<S> {
  pub(super) fn parse_yaml_block_mapping(&mut self) -> GreenNode {
    todo!()
  }

  pub(super) fn parse_yaml_mapping_entry(&mut self) -> GreenNode {
    todo!()
  }

  pub(super) fn parse_yaml_value(&mut self) -> GreenNode {
    todo!()
  }
}

impl<S: Utf8Stream> ParseCtx<S> {
  /// YAML should end when encounter:
  /// - EOF
  /// - Triple dash at indent level 0
  fn should_end_yaml_frontmatter(&mut self) -> bool {
    let PeekYamlResult(result, indent_depth) = self
      .lex_ctx
      .peek_yaml(SKIP_NEWLINE | SKIP_COMMENT | SKIP_STANDALONE_WS | SKIP_TRAILING_WS);

    match result.token.kind() {
      SyntaxKind::Eof => true,
      SyntaxKind::YamlOp if result.token.text().collect::<String>() == "---" => indent_depth == 0,
      _ => false,
    }
  }

  /// YAML expression should end when encounter:
  /// - EOF
  /// - Triple dash at indent level 0
  /// - Dedent
  pub(super) fn should_end_yaml_expr(&mut self) -> bool {
    let PeekYamlResult(result, indent_depth) = self
      .lex_ctx
      .peek_yaml(SKIP_NEWLINE | SKIP_COMMENT | SKIP_STANDALONE_WS | SKIP_TRAILING_WS);

    match result.token.kind() {
      SyntaxKind::Eof => true,
      SyntaxKind::YamlOp if result.token.text().collect::<String>() == "---" => indent_depth == 0,
      SyntaxKind::YamlDedent => true,
      _ => false,
    }
  }
}
