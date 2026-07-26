//! YAML frontmatter parsing

use crate::syntax::diagnostic::Diagnostic;
use typedown_types::stream::Utf8Stream;

use super::constants::*;
use super::ctx::ParseCtx;
use super::ctx::expr_ctx::ExprCtx;
use crate::syntax::green::{GreenNode, SyntaxToken};
use crate::syntax::lex::ctx::LexMode;
use crate::syntax::syntax_kind::SyntaxKind;

// Top-level YAML frontmatter parsing
impl<S: Utf8Stream> ParseCtx<S> {
  /* Top-level YAML frontmatter */

  pub(in crate::syntax::parse) fn parse_yaml_frontmatter(&mut self) -> GreenNode {
    debug_assert!(
      self.lex_ctx.mode() == LexMode::YamlFrontmatter,
      "[ParseCtx::parse_yaml_frontmatter] Lex mode must be YamlFrontmatter"
    );

    let mut children = vec![];

    // Consume opening ---
    let ok = self.consume_yaml_if(
      &mut children,
      SKIP_INDENT,
      |token| token.kind() == SyntaxKind::YamlOp && token.chars().collect::<String>() == "---",
      Diagnostic::MissingFrontmatterMarker {
        offset: self.offset(),
      },
    );
    if !ok {
      self.synchronize_to_triple_dash(&mut children);
    }

    // Expect newline after opening ---
    self.expect_end_of_line(
      &mut children,
      Diagnostic::UnexpectedTokensOnFrontmatterMarkerLine {
        start_offset: self.offset(),
        end_offset: self.offset(),
      },
    );

    // Parse body
    let early_exit = self.parse_yaml_body(&mut children);

    if let Some(ctx) = early_exit
      && ctx != ExprCtx::YamlFrontmatter
    {
      self
        .diagnostics
        .push(Diagnostic::UnexpectedTokensOnFrontmatterMarkerLine {
          start_offset: self.offset(),
          end_offset: self.offset(),
        });
    }

    // Consume closing ---
    // Require the indentation to be 0

    let start_offset = self.offset();
    self.consume_yaml_if(
      &mut children,
      SKIP_NEWLINE | SKIP_WS | SKIP_INDENT,
      |token| token.kind() == SyntaxKind::YamlOp && token.chars().collect::<String>() == "---",
      Diagnostic::MissingFrontmatterMarker {
        offset: start_offset,
      },
    );

    // Expect newline after closing ---
    self.expect_end_of_line(
      &mut children,
      Diagnostic::UnexpectedTokensOnFrontmatterMarkerLine {
        start_offset: self.offset(),
        end_offset: self.offset(),
      },
    );

    self.emit(SyntaxKind::YamlFrontmatter, &children)
  }

  /// Error recovery: skip tokens until `---` or EOF is found.
  fn synchronize_to_triple_dash(&mut self, children: &mut Vec<GreenNode>) {
    let mut error_children = vec![];

    loop {
      let result = self.lex_ctx.lex();
      let kind = result.token.kind();

      let is_target = (kind == SyntaxKind::YamlOp
        && result.token.chars().collect::<String>() == "---")
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
  pub(in crate::syntax::parse) fn parse_yaml_body(
    &mut self,
    children: &mut Vec<GreenNode>,
  ) -> Option<ExprCtx> {
    self.expr_ctx_stack.enter(ExprCtx::YamlFrontmatter);

    let early_exit = if !self.should_end_yaml_frontmatter() {
      let (mapping, early_exit) = self.parse_block_mapping_lit(vec![], 0);
      children.push(mapping);
      early_exit
    } else {
      None
    };

    self.expr_ctx_stack.exit(ExprCtx::YamlFrontmatter);
    early_exit
  }
}

impl<S: Utf8Stream> ParseCtx<S> {
  /// Whether the token is EOF or a YamlIndent with less than the current block indent.
  pub(in crate::syntax::parse) fn is_block_dedent(
    &self,
    token: &SyntaxToken,
    block_indent: usize,
  ) -> bool {
    match token.kind() {
      SyntaxKind::Eof => true,
      SyntaxKind::YamlIndent => token.chars().count() < block_indent,
      _ => false,
    }
  }

  /// YAML should end when encounter:
  /// - EOF
  /// - Triple dash at indent level 0
  fn should_end_yaml_frontmatter(&mut self) -> bool {
    let peek = self
      .lex_ctx
      .peek_yaml(SKIP_NEWLINE | SKIP_COMMENT | SKIP_WS | SKIP_INDENT);

    match peek.token.kind() {
      SyntaxKind::Eof => true,
      SyntaxKind::YamlOp if peek.token.chars().collect::<String>() == "---" => {
        peek.block_indent == 0
      }
      _ => false,
    }
  }
}
