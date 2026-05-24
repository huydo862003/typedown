//! Markdown body parsing

use typedown_types::{stream::Utf8Stream, syntax_kind::SyntaxKind};

use super::ctx::ParseCtx;
use super::ctx::expr_ctx::ExprCtx;
use crate::green::GreenNode;
use crate::lex::ctx::LexMode;

// Markdown body parsing
// We distinguish between block elements and inline elements
// Inline elements (like links) must always be nested in a block element, such as paragraphs
impl<S: Utf8Stream> ParseCtx<S> {
  pub(in crate::parse) fn parse_markdown_body(&mut self) -> GreenNode {
    debug_assert!(
      self.lex_ctx.mode() == LexMode::MarkdownBody,
      "[ParseCtx::parse_markdown_body] Lex mode must be MarkdownBody"
    );
    let children = vec![];
    self.expr_ctx_stack.enter(ExprCtx::MarkdownBody);
    self.expr_ctx_stack.exit(ExprCtx::MarkdownBody);
    self.emit(SyntaxKind::Body, &children)
  }

  /// Parse a heading: `# ...`, `## ...`, etc.
  pub(in crate::parse) fn parse_heading(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse a paragraph: consecutive non-blank text lines.
  pub(in crate::parse) fn parse_paragraph(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse a blockquote: `> ...`.
  pub(in crate::parse) fn parse_blockquote(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse a table: `| ... | ... |`.
  pub(in crate::parse) fn parse_table(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse a table row.
  pub(in crate::parse) fn parse_table_row(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse a table cell.
  pub(in crate::parse) fn parse_table_cell(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse a bullet list: `- ...` or `* ...`.
  pub(in crate::parse) fn parse_bullet_list(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse a bullet list item.
  pub(in crate::parse) fn parse_bullet_list_item(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse an ordered list: `1. ...`.
  pub(in crate::parse) fn parse_ordered_list(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse an ordered list item.
  pub(in crate::parse) fn parse_ordered_list_item(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse a toggle list: `>- ...`.
  pub(in crate::parse) fn parse_toggle_list(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse a toggle list item.
  pub(in crate::parse) fn parse_toggle_list_item(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse a callout block: `::: label ... :::`.
  pub(in crate::parse) fn parse_callout_block(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse a footnote block: `:::footnote ... :::`.
  pub(in crate::parse) fn parse_footnote_block(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse a bibliography block: `:::bibtex ... :::`.
  pub(in crate::parse) fn parse_bibliography_block(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse a link: `[text](url)`.
  pub(in crate::parse) fn parse_link(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse a media embed: `![alt](src)`.
  pub(in crate::parse) fn parse_media(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse a footnote reference: `[^key]`.
  pub(in crate::parse) fn parse_footnote_ref(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse a citation: `[@key]`.
  pub(in crate::parse) fn parse_citation(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse bold text: `**text**`.
  pub(in crate::parse) fn parse_bold(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse italic text: `*text*` or `_text_`.
  pub(in crate::parse) fn parse_italic(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse bold+italic text: `***text***`.
  pub(in crate::parse) fn parse_bold_italic(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse strikethrough text: `~~text~~`.
  pub(in crate::parse) fn parse_strikethrough(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }

  /// Parse a text run: consecutive plain text.
  pub(in crate::parse) fn parse_text(&mut self, current_indent: usize) -> GreenNode {
    todo!()
  }
}
