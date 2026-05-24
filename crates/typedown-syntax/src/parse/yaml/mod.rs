//! YAML frontmatter parsing

use typedown_types::{diagnostic::Diagnostic, stream::Utf8Stream};

use super::constants::*;
use super::ctx::ParseCtx;
use super::ctx::expr_ctx::ExprCtx;
use crate::green::GreenNode;
use crate::lex::ctx::LexMode;
use typedown_types::syntax_kind::SyntaxKind;

// Top-level YAML frontmatter parsing
impl<S: Utf8Stream> ParseCtx<S> {
  /* Top-level YAML frontmatter */

  pub(in crate::parse) fn parse_yaml_frontmatter(&mut self) -> GreenNode {
    debug_assert!(
      self.lex_ctx.mode() == LexMode::YamlFrontmatter,
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
    let early_exit = self.parse_yaml_body(&mut children);

    if let Some(ctx) = early_exit {
      if ctx != ExprCtx::YamlFrontmatter {
        self
          .diagnostics
          .push(Diagnostic::UnexpectedTokensOnFrontmatterMarkerLine {
            start_offset: self.offset(),
            end_offset: self.offset(),
          });
      }
    }

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
  pub(in crate::parse) fn parse_yaml_body(
    &mut self,
    children: &mut Vec<GreenNode>,
  ) -> Option<ExprCtx> {
    self.expr_ctx_stack.enter(ExprCtx::YamlFrontmatter);

    let early_exit = if !self.should_end_yaml_frontmatter() {
      let (mapping, early_exit) = self.parse_block_mapping_lit();
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
  /// YAML should end when encounter:
  /// - EOF
  /// - Triple dash at indent level 0
  fn should_end_yaml_frontmatter(&mut self) -> bool {
    let peek = self
      .lex_ctx
      .peek_yaml(SKIP_NEWLINE | SKIP_COMMENT | SKIP_STANDALONE_WS | SKIP_TRAILING_WS);

    match peek.result.token.kind() {
      SyntaxKind::Eof => true,
      SyntaxKind::YamlOp if peek.result.token.text().collect::<String>() == "---" => {
        peek.indent_depth == 0
      }
      _ => false,
    }
  }
}
