//! YAML frontmatter parsing

use typedown_types::{diagnostic::Diagnostic, stream::Utf8Stream};

use super::constants::*;
use super::ctx::ParseCtx;
use crate::green::{GreenNode, syntax_kind::SyntaxKind};
use crate::lex::ctx::LexMode;

// YAML frontmatter parsing
impl<S: Utf8Stream> ParseCtx<S> {
  pub(super) fn parse_yaml_frontmatter(&mut self) -> GreenNode {
    debug_assert!(
      *self.lex_ctx.mode() == LexMode::YamlFrontmatter,
      "[ParseCtx::parse_yaml_frontmatter] Lex mode must be YamlFrontmatter"
    );

    let mut children = vec![];

    // Consume opening ---
    let ok = self.consume_if(
      &mut children,
      SKIP_COMMENT,
      |token| token.kind() == SyntaxKind::YamlOp && token.text().collect::<String>() == "---",
      Diagnostic::MissingFrontmatterMarker {
        offset: self.offset(),
      },
    );
    if !ok {
      self.synchronize_to_triple_dash(&mut children);
    }
    self.expect_line_end(&mut children);

    // TODO: parse frontmatter content

    // Skip to the beginning of next line
    self.advance_to_next_line(&mut children);
    // Consume closing ---
    self.consume_if(
      &mut children,
      SKIP_NEWLINE,
      |token| token.kind() == SyntaxKind::YamlOp && token.text().collect::<String>() == "---",
      Diagnostic::MissingFrontmatterMarker {
        offset: self.offset(),
      },
    );
    self.expect_line_end(&mut children);

    self.emit(SyntaxKind::Frontmatter, &children)
  }

  /// After consuming `---`, expect only whitespace/comments until end of line.
  /// Any other tokens are wrapped in an Error node with a diagnostic.
  fn expect_line_end(&mut self, children: &mut Vec<GreenNode>) {
    let mut error_children = vec![];
    let start_offset = self.offset();

    loop {
      let result = self.lex_ctx.lex();
      match result.token.kind() {
        SyntaxKind::Whitespace | SyntaxKind::YamlComment => {
          children.push(GreenNode::from_token(result.token));
        }
        SyntaxKind::Newline | SyntaxKind::Eof => {
          children.push(GreenNode::from_token(result.token));
          break;
        }
        _ => {
          error_children.push(GreenNode::from_token(result.token));
        }
      }
    }

    if !error_children.is_empty() {
      children.push(self.emit(SyntaxKind::Error, &error_children));
      self.diagnostics.push(Diagnostic::ExtraTokensAfterFrontmatterMarker {
        start_offset,
        end_offset: self.offset(),
      });
    }
  }

  /// Error recovery: skip tokens until `---` or EOF is found.
  fn synchronize_to_triple_dash(&mut self, children: &mut Vec<GreenNode>) {
    let mut error_children = vec![];

    loop {
      let result = self.lex_ctx.lex();
      let kind = result.token.kind();

      let is_target = (kind == SyntaxKind::YamlOp && result.token.text().collect::<String>() == "---")
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
}
