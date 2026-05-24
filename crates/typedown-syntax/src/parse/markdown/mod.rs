//! Markdown body parsing

use typedown_types::diagnostic::Diagnostic;
use typedown_types::{stream::Utf8Stream, syntax_kind::SyntaxKind};

use super::ctx::ParseCtx;
use super::ctx::expr_ctx::ExprCtx;
use crate::green::{GreenNode, SyntaxToken};
use crate::lex::ctx::LexMode;
use crate::parse::constants::SKIP_NONE;

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
  /// INVARIANT: The next token should be a hash sequence  /// Any whitespaces must be consumed by the parent to pass the correct current_indent
  pub(in crate::parse) fn parse_heading(&mut self, current_indent: usize) -> GreenNode {
    fn is_hash(token: &SyntaxToken) -> bool {
      token.kind() == SyntaxKind::MdSymbol && token.text().all(|c| c == '#')
    }
    debug_assert!(
      is_hash(&self.lex_ctx.peek_md(SKIP_NONE).token),
      "[ParseCtx::parse_heading] Expect the next immediate token to be a hash"
    );
    let mut children = vec![];

    self.consume_md_if(
      &mut children,
      SKIP_NONE,
      is_hash,
      Diagnostic::MissingMarkdownHeadingHash {
        start_offset: self.offset(),
        end_offset: self.offset(),
      },
    );

    let next_token = &self.lex_ctx.peek_md(SKIP_NONE).token;
    if next_token.kind() != SyntaxKind::Whitespace {
      self.emit_diagnostic(Diagnostic::MissingRequiredSpacesBetweenHashAndHeading {
        start_offset: self.offset(),
        end_offset: self.offset(),
      });
    } else {
      self.advance_md(&mut children, SKIP_NONE);
    }

    todo!();
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
