//! YAML frontmatter parsing

use typedown_types::stream::Utf8Stream;

use super::ctx::ParseCtx;
use crate::green::GreenNode;
use crate::lex::ctx::LexMode;

// YAML frontmatter parsing
impl<S: Utf8Stream> ParseCtx<S> {
  pub(super) fn parse_yaml_frontmatter(&mut self) -> GreenNode {
    debug_assert!(
      *self.lex_ctx.mode() == LexMode::YamlFrontmatter,
      "[ParseCtx::parse_yaml_frontmatter] Lex mode must be YamlFrontmatter"
    );
    todo!()
  }
}
