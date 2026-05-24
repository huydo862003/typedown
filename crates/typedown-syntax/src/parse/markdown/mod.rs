//! Markdown body parsing

use typedown_types::stream::Utf8Stream;

use super::ctx::ParseCtx;
use crate::green::GreenNode;
use crate::lex::ctx::LexMode;

// Markdown body parsing
impl<S: Utf8Stream> ParseCtx<S> {
  pub(in crate::parse) fn parse_markdown_body(&mut self) -> GreenNode {
    debug_assert!(
      self.lex_ctx.mode() == LexMode::MarkdownBody,
      "[ParseCtx::parse_markdown_body] Lex mode must be MarkdownBody"
    );
    todo!()
  }
}
