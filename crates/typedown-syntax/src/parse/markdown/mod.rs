//! Markdown body parsing

use typedown_types::diagnostic::Diagnostic;
use typedown_types::{stream::Utf8Stream, syntax_kind::SyntaxKind};

use super::ctx::ParseCtx;
use super::ctx::expr_ctx::ExprCtx;
use crate::green::{GreenNode, SyntaxToken};
use crate::lex::ctx::LexMode;
use crate::parse::constants::{SKIP_LEADING_WS, SKIP_NONE, SKIP_TRAILING_WS};

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

    todo!();

    self.expr_ctx_stack.exit(ExprCtx::MarkdownBody);
    self.emit(SyntaxKind::Body, &children)
  }

  pub(in crate::parse) fn parse_md_block_element(&mut self) -> GreenNode {
    todo!()
  }

  pub(in crate::parse) fn parse_md_inline_element(&mut self) -> GreenNode {
    todo!()
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

    // Require at least one inline element
    let has_inline = {
      let next = self.lex_ctx.peek_md(SKIP_TRAILING_WS);
      !matches!(next.token.kind(), SyntaxKind::Newline | SyntaxKind::Eof)
    };
    if !has_inline {
      self.emit_diagnostic(Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Text,
        start_offset: self.offset(),
        end_offset: self.offset(),
      });
    } else {
      // Parse inline elements until newline or EOF
      loop {
        let next_kind = self.lex_ctx.peek_md(SKIP_NONE).token.kind();
        if matches!(next_kind, SyntaxKind::Newline | SyntaxKind::Eof) {
          break;
        }
        let inline = self.parse_md_inline_element();
        children.push(inline);
      }
    }

    // Consume the trailing newline if present
    let next_kind = self.lex_ctx.peek_md(SKIP_NONE).token.kind();
    if next_kind == SyntaxKind::Newline {
      self.advance_md(&mut children, SKIP_NONE);
    }

    self.emit(SyntaxKind::Heading, &children)
  }

  /// Parse a paragraph: consecutive non-blank text lines.
  /// INVARIANT: The current line is not blank (caller must ensure there is content).
  pub(in crate::parse) fn parse_paragraph(&mut self, current_indent: usize) -> GreenNode {
    let mut children = vec![];

    loop {
      // Parse all inline elements on this line
      loop {
        let next_kind = self.lex_ctx.peek_md(SKIP_NONE).token.kind();
        if matches!(next_kind, SyntaxKind::Newline | SyntaxKind::Eof) {
          break;
        }
        let inline = self.parse_md_inline_element();
        children.push(inline);
      }

      // Stop at EOF
      let next_kind = self.lex_ctx.peek_md(SKIP_NONE).token.kind();
      if next_kind == SyntaxKind::Eof {
        break;
      }

      // Consume the newline
      self.advance_md(&mut children, SKIP_NONE);

      // Peek past leading whitespace on the next line to decide whether to continue
      let next = self.lex_ctx.peek_md(SKIP_LEADING_WS);
      if matches!(next.token.kind(), SyntaxKind::Newline | SyntaxKind::Eof) {
        // Blank line: end paragraph
        break;
      }
      if self.is_md_block_start() {
        if next.indent_depth > current_indent {
          // Indented block: parse as a nested child of this paragraph
          let block = self.parse_md_block_element();
          children.push(block);
        } else {
          // Block at same or lower indent: end paragraph
          break;
        }
      }
    }

    self.emit(SyntaxKind::Paragraph, &children)
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

  /// Whether the next non-leading-whitespace token starts a block-level element.
  /// INVARIANT: Must be called right after consuming a newline.
  fn is_md_block_start(&mut self) -> bool {
    let next = self.lex_ctx.peek_md(SKIP_LEADING_WS);
    if next.token.kind() != SyntaxKind::MdSymbol {
      // Number can start an ordered list item: `1. ...`
      return next.token.kind() == SyntaxKind::Number;
    }
    let text: String = next.token.text().collect();
    let first = match text.chars().next() {
      Some(char) => char,
      None => return false,
    };
    matches!(
      first,
      '#'  // heading
      | '-' | '*' | '+' // bullet list
      | '>' // blockquote or toggle list
      | '|' // table (this can introduces some false positives, but I think it's okay)
      | ':' // callout/footnote/bibliography (:::)
    )
  }
}
