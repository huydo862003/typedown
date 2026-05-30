//! Markdown body parsing

use typedown_types::diagnostic::Diagnostic;
use typedown_types::{stream::Utf8Stream, syntax_kind::SyntaxKind};

use super::ctx::ParseCtx;
use super::ctx::expr_ctx::ExprCtx;
use crate::green::{GreenNode, SyntaxToken};
use crate::lex::ctx::LexMode;
use crate::parse::constants::{SKIP_NEWLINE, SKIP_NONE, SKIP_WS};

// Markdown body parsing
// We distinguish between block elements and inline elements
// Inline elements (like links) must always be nested in a block element, such as paragraphs
impl<S: Utf8Stream> ParseCtx<S> {
  pub(in crate::parse) fn parse_markdown_body(&mut self) -> GreenNode {
    debug_assert!(
      self.lex_ctx.mode() == LexMode::MarkdownBody,
      "[ParseCtx::parse_markdown_body] Lex mode must be MarkdownBody"
    );
    let mut children = vec![];
    self.expr_ctx_stack.enter(ExprCtx::MarkdownBody);

    loop {
      // Skip blank lines
      while self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::Newline {
        self.advance_md(&mut children, SKIP_NONE);
      }

      if self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::Eof {
        break;
      }

      let (block, early_exit) = self.parse_md_block_element();
      children.push(block);

      if early_exit == Some(ExprCtx::MarkdownBody) {
        // Consume erroneous tokens until EOF
        let mut error_children = vec![];
        loop {
          let next = self.lex_ctx.peek_md(SKIP_NONE);
          if next.token.kind() == SyntaxKind::Eof {
            break;
          }
          self.advance_md(&mut error_children, SKIP_NONE);
        }
        if !error_children.is_empty() {
          children.push(self.emit(SyntaxKind::Error, &error_children));
        }
        break;
      }
      if early_exit.is_some() {
        // Unexpected early exit from a child: consume as error
        let mut error_children = vec![];
        loop {
          let next = self.lex_ctx.peek_md(SKIP_NONE);
          if next.token.kind() == SyntaxKind::Eof {
            break;
          }
          self.advance_md(&mut error_children, SKIP_NONE);
        }
        if !error_children.is_empty() {
          children.push(self.emit(SyntaxKind::Error, &error_children));
        }
        break;
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::MarkdownBody);
    self.emit(SyntaxKind::Body, &children)
  }

  /// Parse a block-level element.
  /// INVARIANT: Must be at start of line with prefix already consumed.
  /// INVARIANT: Block elements do not consume their trailing newline as one newline can end multiple block elements
  pub(in crate::parse) fn parse_md_block_element(&mut self) -> (GreenNode, Option<ExprCtx>) {
    debug_assert!(
      self.lex_ctx.mode() == LexMode::MarkdownBody,
      "[ParseCtx::parse_md_block_element] Lex mode must be MarkdownBody"
    );

    let next = self.lex_ctx.peek_md(SKIP_NONE);
    match next.token.kind() {
      SyntaxKind::Eof => {
        let mut children = vec![];
        self.advance_md(&mut children, SKIP_NONE);
        (self.emit(SyntaxKind::Error, &children), None)
      }
      SyntaxKind::Newline => {
        // Blank line: consume and return empty
        let mut children = vec![];
        self.advance_md(&mut children, SKIP_NONE);
        (self.emit(SyntaxKind::Text, &children), None)
      }
      _ if self.is_heading_start(SKIP_NONE) => self.parse_heading(),
      _ if self.is_toggle_list_start(SKIP_NONE) => self.parse_toggle_list(),
      _ if self.is_blockquote_start(SKIP_NONE) => self.parse_blockquote(),
      _ if self.is_bullet_list_start(SKIP_NONE) => self.parse_bullet_list(),
      _ if self.is_ordered_list_start(SKIP_NONE) => self.parse_ordered_list(),
      _ if self.is_table_start(SKIP_NONE) => self.parse_table(),
      _ if self.is_callout_start(SKIP_NONE) => self.parse_callout_block(),
      _ if self.is_media_block_start(SKIP_NONE) => self.parse_media(),
      _ if self.is_code_or_math_block_start(SKIP_NONE) => {
        let mut children = vec![];
        let kind = next.token.kind();
        self.advance_md(&mut children, SKIP_NONE);
        (self.emit(kind, &children), None)
      }
      _ => self.parse_paragraph(),
    }
  }

  /// Parse an inline element.
  /// INVARIANT: Must not be at a Newline or EOF.
  pub(in crate::parse) fn parse_md_inline_element(&mut self) -> (GreenNode, Option<ExprCtx>) {
    debug_assert!(
      !matches!(
        self.lex_ctx.peek_md(SKIP_NONE).token.kind(),
        SyntaxKind::Newline | SyntaxKind::Eof
      ),
      "[ParseCtx::parse_md_inline_element] Must not be at Newline or EOF"
    );

    let next = self.lex_ctx.peek_md(SKIP_NONE);
    match next.token.kind() {
      SyntaxKind::LBracket => {
        // Check for footnote ref `[^`, citation `[@`, or link `[`
        let second = self.lex_ctx.peek_md_nth(1, SKIP_NONE);
        if second.token.kind() == SyntaxKind::MdSymbol {
          let text: String = second.token.text().collect();
          if text == "^" {
            return self.parse_footnote_ref();
          }
          if text == "@" {
            return self.parse_citation();
          }
        }
        self.parse_link()
      }
      SyntaxKind::MdSymbol => {
        let text: String = next.token.text().collect();
        match text.as_str() {
          "***" => self.parse_bold_italic(),
          "**" => self.parse_bold(),
          "*" | "_" => self.parse_italic(),
          "~~" => self.parse_strikethrough(),
          "!" => {
            let second = self.lex_ctx.peek_md_nth(1, SKIP_NONE);
            if second.token.kind() == SyntaxKind::LBracket {
              self.parse_media()
            } else {
              self.parse_text()
            }
          }
          _ => self.parse_text(),
        }
      }
      SyntaxKind::InterpStart => {
        let (fragment, early_exit) = self.parse_interp_fragment(0);
        (fragment, early_exit)
      }
      SyntaxKind::InlineMath | SyntaxKind::InlineCode => {
        // These are already lexed as single tokens
        let mut children = vec![];
        self.advance_md(&mut children, SKIP_NONE);
        (self.emit(SyntaxKind::Text, &children), None)
      }
      _ => self.parse_text(),
    }
  }

  /// Parse a heading: `# ...`, `## ...`, etc.
  /// INVARIANT: The next token should be a hash sequence
  pub(in crate::parse) fn parse_heading(&mut self) -> (GreenNode, Option<ExprCtx>) {
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
      let next = self.lex_ctx.peek_md(SKIP_WS);
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
        let (inline, early_exit) = self.parse_md_inline_element();
        children.push(inline);
        if early_exit.is_some() {
          return (self.emit(SyntaxKind::Heading, &children), early_exit);
        }
      }
    }

    (self.emit(SyntaxKind::Heading, &children), None)
  }

  /// Parse a paragraph: consecutive non-blank text lines.
  /// INVARIANT: The current line is not blank (caller must ensure there is content).
  pub(in crate::parse) fn parse_paragraph(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mut children = vec![];

    loop {
      // Parse all inline elements on this line
      loop {
        let next_kind = self.lex_ctx.peek_md(SKIP_NONE).token.kind();
        if matches!(next_kind, SyntaxKind::Newline | SyntaxKind::Eof) {
          break;
        }
        let (inline, early_exit) = self.parse_md_inline_element();
        children.push(inline);
        if early_exit.is_some() {
          return (self.emit(SyntaxKind::Paragraph, &children), early_exit);
        }
      }

      // Stop at EOF
      let next_kind = self.lex_ctx.peek_md(SKIP_NONE).token.kind();
      if next_kind == SyntaxKind::Eof {
        break;
      }

      // Peek past the newline to decide whether to continue
      let prefix_len = self.expr_ctx_stack.md_prefix_tokens().len();
      let after_prefix = self.lex_ctx.peek_md_nth(prefix_len, SKIP_NEWLINE);
      if matches!(
        after_prefix.token.kind(),
        SyntaxKind::Newline | SyntaxKind::Eof
      ) {
        break;
      }
      {
        let expected = self.expr_ctx_stack.md_prefix_tokens().to_vec();
        let prefix_ok = expected.iter().enumerate().all(|(idx, expected_token)| {
          self.lex_ctx.peek_md_nth(idx, SKIP_NEWLINE).token == *expected_token
        });
        if !prefix_ok {
          break;
        }
      }
      if self.is_md_any_block_start(SKIP_NEWLINE) {
        break;
      }

      // Paragraph continues onto the next line, consume the newline
      self.advance_md(&mut children, SKIP_NONE);
    }

    (self.emit(SyntaxKind::Paragraph, &children), None)
  }

  /// Parse a blockquote: `> ...`.
  /// INVARIANT: Expect the next token to be `>`
  pub(in crate::parse) fn parse_blockquote(&mut self) -> (GreenNode, Option<ExprCtx>) {
    debug_assert!(
      self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::MdSymbol
        && self
          .lex_ctx
          .peek_md(SKIP_NONE)
          .token
          .text()
          .collect::<String>()
          == ">",
      "[ParseCtx::parse_blockquote] Expected >"
    );

    let mut children = vec![];

    self.expr_ctx_stack.enter(ExprCtx::MdBlockQuote);

    // Consume `>`
    self.advance_md(&mut children, SKIP_NONE);

    // Require a space after `>`
    if self.lex_ctx.peek_md(SKIP_NONE).token.kind() != SyntaxKind::Whitespace {
      self.emit_diagnostic(Diagnostic::MissingRequiredSpacesBetweenHashAndHeading {
        start_offset: self.offset(),
        end_offset: self.offset(),
      });
    } else {
      self.advance_md(&mut children, SKIP_NONE);
    }

    // Parse block elements until the blockquote ends
    loop {
      let next_kind = self.lex_ctx.peek_md(SKIP_NONE).token.kind();
      if next_kind == SyntaxKind::Eof {
        break;
      }
      if matches!(next_kind, SyntaxKind::Newline) {
        if !self.peek_md_newline_and_prefix() {
          break;
        }
        self.consume_md_newline_and_prefix(&mut children);
        continue;
      }

      let (block, early_exit) = self.parse_md_block_element();
      children.push(block);
      if early_exit.is_some_and(|ctx| ctx != ExprCtx::MdBlockQuote) {
        self.expr_ctx_stack.exit(ExprCtx::MdBlockQuote);
        return (self.emit(SyntaxKind::Blockquote, &children), early_exit);
      }
      if early_exit == Some(ExprCtx::MdBlockQuote) {
        break;
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::MdBlockQuote);
    (self.emit(SyntaxKind::Blockquote, &children), None)
  }

  /// Parse a table: `| ... | ... |`.
  /// INVARIANT: Next token must be MdSymbol `|`.
  pub(in crate::parse) fn parse_table(&mut self) -> (GreenNode, Option<ExprCtx>) {
    debug_assert!(
      self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::MdSymbol
        && self
          .lex_ctx
          .peek_md(SKIP_NONE)
          .token
          .text()
          .collect::<String>()
          == "|",
      "[ParseCtx::parse_table] Expected |"
    );

    let mut children = vec![];

    self.expr_ctx_stack.enter(ExprCtx::MdTable);

    // Parse header row
    let (row, col_count, early_exit) = self.parse_table_row();
    let expected_cols = col_count;
    children.push(row);
    if early_exit.is_some_and(|ctx| ctx != ExprCtx::MdTable) {
      self.expr_ctx_stack.exit(ExprCtx::MdTable);
      return (self.emit(SyntaxKind::Table, &children), early_exit);
    }

    // Parse required separator row
    let sep_start = self.offset();
    if self.lex_ctx.peek_md(SKIP_NONE).token.kind() != SyntaxKind::Newline
      || !self.consume_md_newline_and_prefix(&mut children)
    {
      self.emit_diagnostic(Diagnostic::MissingTableSeparatorRow {
        start_offset: sep_start,
        end_offset: self.offset(),
      });
      self.expr_ctx_stack.exit(ExprCtx::MdTable);
      return (self.emit(SyntaxKind::Table, &children), None);
    }
    // Verify separator row starts with `|` followed by `-`
    let next = self.lex_ctx.peek_md(SKIP_NONE);
    let next2 = self.lex_ctx.peek_md_nth(1, SKIP_WS);
    let is_separator = next.token.kind() == SyntaxKind::MdSymbol
      && next.token.text().collect::<String>() == "|"
      && next2.token.kind() == SyntaxKind::MdSymbol
      && next2.token.text().collect::<String>().starts_with('-');
    if !is_separator {
      self.emit_diagnostic(Diagnostic::MissingTableSeparatorRow {
        start_offset: sep_start,
        end_offset: self.offset(),
      });
      self.expr_ctx_stack.exit(ExprCtx::MdTable);
      return (self.emit(SyntaxKind::Table, &children), None);
    }
    let (sep, early_exit) = self.parse_table_separator_row();
    children.push(sep);
    if early_exit.is_some_and(|ctx| ctx != ExprCtx::MdTable) {
      self.expr_ctx_stack.exit(ExprCtx::MdTable);
      return (self.emit(SyntaxKind::Table, &children), early_exit);
    }

    // Parse body rows
    loop {
      if self.lex_ctx.peek_md(SKIP_NONE).token.kind() != SyntaxKind::Newline {
        break;
      }
      if !self.consume_md_newline_and_prefix(&mut children) {
        break;
      }
      let next = self.lex_ctx.peek_md(SKIP_NONE);
      if next.token.kind() != SyntaxKind::MdSymbol || next.token.text().collect::<String>() != "|" {
        break;
      }

      let row_start = self.offset();
      let (row, col_count, early_exit) = self.parse_table_row();
      if col_count != expected_cols {
        self.emit_diagnostic(Diagnostic::TableColumnCountMismatch {
          expected: expected_cols,
          found: col_count,
          start_offset: row_start,
          end_offset: self.offset(),
        });
      }
      children.push(row);
      if early_exit.is_some_and(|ctx| ctx != ExprCtx::MdTable) {
        self.expr_ctx_stack.exit(ExprCtx::MdTable);
        return (self.emit(SyntaxKind::Table, &children), early_exit);
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::MdTable);
    (self.emit(SyntaxKind::Table, &children), None)
  }

  /// Parse a table row: `| cell | cell |`.
  /// Returns the node, cell count, and early exit context.
  /// INVARIANT: Next token must be MdSymbol `|`.
  fn parse_table_row(&mut self) -> (GreenNode, usize, Option<ExprCtx>) {
    debug_assert!(
      self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::MdSymbol
        && self
          .lex_ctx
          .peek_md(SKIP_NONE)
          .token
          .text()
          .collect::<String>()
          == "|",
      "[ParseCtx::parse_table_row] Expected |"
    );

    let mut children = vec![];
    let mut cell_count = 0;

    self.expr_ctx_stack.enter(ExprCtx::MdTableRow);

    // Consume leading `|`
    self.advance_md(&mut children, SKIP_NONE);

    loop {
      // Check for end of row
      let next = self.lex_ctx.peek_md(SKIP_NONE);
      if matches!(next.token.kind(), SyntaxKind::Newline | SyntaxKind::Eof) {
        break;
      }

      // Parse a cell
      let (cell, early_exit) = self.parse_table_cell();
      children.push(cell);
      cell_count += 1;
      if early_exit.is_some_and(|ctx| ctx != ExprCtx::MdTableRow) {
        self.expr_ctx_stack.exit(ExprCtx::MdTableRow);
        return (
          self.emit(SyntaxKind::TableRow, &children),
          cell_count,
          early_exit,
        );
      }

      // Consume `|` separator
      let next = self.lex_ctx.peek_md(SKIP_NONE);
      if next.token.kind() == SyntaxKind::MdSymbol && next.token.text().collect::<String>() == "|" {
        self.advance_md(&mut children, SKIP_NONE);
      } else {
        break;
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::MdTableRow);
    (self.emit(SyntaxKind::TableRow, &children), cell_count, None)
  }

  /// Parse a table separator row: `| --- | --- |`.
  /// INVARIANT: Next token must be MdSymbol `|`.
  fn parse_table_separator_row(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mut children = vec![];

    self.expr_ctx_stack.enter(ExprCtx::MdTableRow);

    // Consume everything until Newline or EOF
    loop {
      let next = self.lex_ctx.peek_md(SKIP_NONE);
      if matches!(next.token.kind(), SyntaxKind::Newline | SyntaxKind::Eof) {
        break;
      }
      self.advance_md(&mut children, SKIP_NONE);
    }

    self.expr_ctx_stack.exit(ExprCtx::MdTableRow);
    (self.emit(SyntaxKind::TableSeparatorRow, &children), None)
  }

  /// Parse a table cell: inline content until `|` or end of line.
  fn parse_table_cell(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mut children = vec![];

    self.expr_ctx_stack.enter(ExprCtx::MdTableCell);

    // Skip leading whitespace
    if self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::Whitespace {
      self.advance_md(&mut children, SKIP_NONE);
    }

    loop {
      let next = self.lex_ctx.peek_md(SKIP_NONE);
      // End on `|`, Newline, or EOF
      if matches!(next.token.kind(), SyntaxKind::Newline | SyntaxKind::Eof) {
        break;
      }
      if next.token.kind() == SyntaxKind::MdSymbol && next.token.text().collect::<String>() == "|" {
        break;
      }

      let (inline, early_exit) = self.parse_md_inline_element();
      children.push(inline);
      if early_exit.is_some_and(|ctx| ctx != ExprCtx::MdTableCell) {
        self.expr_ctx_stack.exit(ExprCtx::MdTableCell);
        return (self.emit(SyntaxKind::TableCell, &children), early_exit);
      }
      if early_exit == Some(ExprCtx::MdTableCell) {
        break;
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::MdTableCell);
    (self.emit(SyntaxKind::TableCell, &children), None)
  }

  /// Parse a bullet list: `- ...` or `* ...` or `+ ...`.
  /// INVARIANT: Must be after prefix. Next token must be MdSymbol `-`, `*`, or `+`.
  pub(in crate::parse) fn parse_bullet_list(&mut self) -> (GreenNode, Option<ExprCtx>) {
    debug_assert!(
      {
        let peek = self.lex_ctx.peek_md(SKIP_NONE);
        peek.token.kind() == SyntaxKind::MdSymbol && {
          let text: String = peek.token.text().collect();
          text == "-" || text == "*" || text == "+"
        }
      },
      "[ParseCtx::parse_bullet_list] Expected -, *, or +"
    );

    let mut children = vec![];
    let bullet: String = self.lex_ctx.peek_md(SKIP_NONE).token.text().collect();

    self.expr_ctx_stack.enter(ExprCtx::MdUnorderedList);

    // Parse first list item
    let (item, early_exit) = self.parse_bullet_list_item(&bullet);
    children.push(item);
    if early_exit.is_some_and(|ctx| ctx != ExprCtx::MdUnorderedList) {
      self.expr_ctx_stack.exit(ExprCtx::MdUnorderedList);
      return (self.emit(SyntaxKind::BulletList, &children), early_exit);
    }

    // Parse remaining list items
    loop {
      // Consume newline + prefix, then check if next line starts another item with the same bullet
      if !self.consume_md_newline_and_prefix(&mut children) {
        break;
      }
      let next = self.lex_ctx.peek_md(SKIP_NONE);
      if next.token.kind() != SyntaxKind::MdSymbol {
        break;
      }
      let text: String = next.token.text().collect();
      if text != bullet {
        break;
      }

      let (item, early_exit) = self.parse_bullet_list_item(&bullet);
      children.push(item);
      if early_exit.is_some_and(|ctx| ctx != ExprCtx::MdUnorderedList) {
        self.expr_ctx_stack.exit(ExprCtx::MdUnorderedList);
        return (self.emit(SyntaxKind::BulletList, &children), early_exit);
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::MdUnorderedList);
    (self.emit(SyntaxKind::BulletList, &children), None)
  }

  /// Parse a bullet list item: `- content` or `* content` or `+ content`.
  /// INVARIANT: Next token must be the bullet marker matching `bullet`.
  fn parse_bullet_list_item(&mut self, bullet: &str) -> (GreenNode, Option<ExprCtx>) {
    debug_assert!(
      {
        let peek = self.lex_ctx.peek_md(SKIP_NONE);
        peek.token.kind() == SyntaxKind::MdSymbol && peek.token.text().collect::<String>() == bullet
      },
      "[ParseCtx::parse_bullet_list_item] Expected bullet marker"
    );

    let mut children = vec![];

    self.expr_ctx_stack.enter(ExprCtx::MdUnorderedListItem);

    // Consume the bullet marker
    self.advance_md(&mut children, SKIP_NONE);

    // Require a space after the bullet
    if self.lex_ctx.peek_md(SKIP_NONE).token.kind() != SyntaxKind::Whitespace {
      self.emit_diagnostic(Diagnostic::MissingRequiredSpacesBetweenHashAndHeading {
        start_offset: self.offset(),
        end_offset: self.offset(),
      });
    } else {
      self.advance_md(&mut children, SKIP_NONE);
    }

    // Parse block elements until end of list item
    loop {
      let next_kind = self.lex_ctx.peek_md(SKIP_NONE).token.kind();
      if next_kind == SyntaxKind::Eof {
        break;
      }
      if next_kind == SyntaxKind::Newline {
        if !self.peek_md_newline_and_prefix() {
          break;
        }
        self.consume_md_newline_and_prefix(&mut children);
        let next = self.lex_ctx.peek_md(SKIP_NONE);
        if matches!(next.token.kind(), SyntaxKind::Newline | SyntaxKind::Eof) {
          break;
        }
        // Same-level bullet starts a new list item
        if next.token.kind() == SyntaxKind::MdSymbol
          && next.token.text().collect::<String>() == bullet
        {
          break;
        }
        continue;
      }

      let (block, early_exit) = self.parse_md_block_element();
      children.push(block);
      if early_exit.is_some_and(|ctx| ctx != ExprCtx::MdUnorderedListItem) {
        self.expr_ctx_stack.exit(ExprCtx::MdUnorderedListItem);
        return (self.emit(SyntaxKind::BulletListItem, &children), early_exit);
      }
      if early_exit == Some(ExprCtx::MdUnorderedListItem) {
        break;
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::MdUnorderedListItem);
    (self.emit(SyntaxKind::BulletListItem, &children), None)
  }

  /// Parse an ordered list: `1. ...`.
  /// INVARIANT: The next tokens must be MdNumber and MdSymbol dot.
  pub(in crate::parse) fn parse_ordered_list(&mut self) -> (GreenNode, Option<ExprCtx>) {
    debug_assert!(
      self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::MdNumber,
      "[ParseCtx::parse_ordered_list] Expected MdNumber"
    );
    debug_assert!(
      self.lex_ctx.peek_md_nth(1, SKIP_NONE).token.kind() == SyntaxKind::MdSymbol
        && self
          .lex_ctx
          .peek_md_nth(1, SKIP_NONE)
          .token
          .text()
          .collect::<String>()
          == ".",
      "[ParseCtx::parse_ordered_list] Expected . after MdNumber"
    );

    let mut children = vec![];

    self.expr_ctx_stack.enter(ExprCtx::MdOrderedList);

    // Parse first list item
    let (item, early_exit) = self.parse_ordered_list_item();
    children.push(item);
    if early_exit.is_some_and(|ctx| ctx != ExprCtx::MdOrderedList) {
      self.expr_ctx_stack.exit(ExprCtx::MdOrderedList);
      return (self.emit(SyntaxKind::OrderedList, &children), early_exit);
    }

    // Parse remaining list items
    loop {
      // Consume newline + prefix, then check for next item
      if !self.consume_md_newline_and_prefix(&mut children) {
        break;
      }
      let next = self.lex_ctx.peek_md(SKIP_NONE);
      if next.token.kind() != SyntaxKind::MdNumber {
        break;
      }
      // Verify `.` follows the number
      let dot = self.lex_ctx.peek_md_nth(1, SKIP_NONE);
      if dot.token.kind() != SyntaxKind::MdSymbol || dot.token.text().collect::<String>() != "." {
        break;
      }

      let (item, early_exit) = self.parse_ordered_list_item();
      children.push(item);
      if early_exit.is_some_and(|ctx| ctx != ExprCtx::MdOrderedList) {
        self.expr_ctx_stack.exit(ExprCtx::MdOrderedList);
        return (self.emit(SyntaxKind::OrderedList, &children), early_exit);
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::MdOrderedList);
    (self.emit(SyntaxKind::OrderedList, &children), None)
  }

  /// Parse an ordered list item: `1. content`.
  /// INVARIANT: Next token must be MdNumber followed by `.`.
  fn parse_ordered_list_item(&mut self) -> (GreenNode, Option<ExprCtx>) {
    debug_assert!(
      self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::MdNumber,
      "[ParseCtx::parse_ordered_list_item] Expected MdNumber"
    );

    let mut children = vec![];

    self.expr_ctx_stack.enter(ExprCtx::MdOrderedListItem);

    // Consume the number
    self.advance_md(&mut children, SKIP_NONE);

    // Consume `.`
    self.consume_md_if(
      &mut children,
      SKIP_NONE,
      |token| token.kind() == SyntaxKind::MdSymbol && token.text().collect::<String>() == ".",
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::OrderedListItem,
        start_offset: self.offset(),
        end_offset: self.offset(),
      },
    );

    // Require a space after `.`
    if self.lex_ctx.peek_md(SKIP_NONE).token.kind() != SyntaxKind::Whitespace {
      self.emit_diagnostic(Diagnostic::MissingRequiredSpacesBetweenHashAndHeading {
        start_offset: self.offset(),
        end_offset: self.offset(),
      });
    } else {
      self.advance_md(&mut children, SKIP_NONE);
    }

    // Parse block elements until end of list item
    loop {
      let next_kind = self.lex_ctx.peek_md(SKIP_NONE).token.kind();
      if next_kind == SyntaxKind::Eof {
        break;
      }
      if next_kind == SyntaxKind::Newline {
        if !self.peek_md_newline_and_prefix() {
          break;
        }
        self.consume_md_newline_and_prefix(&mut children);
        let next = self.lex_ctx.peek_md(SKIP_NONE);
        if matches!(next.token.kind(), SyntaxKind::Newline | SyntaxKind::Eof) {
          break;
        }
        // Next ordered item starts a new list item
        if next.token.kind() == SyntaxKind::MdNumber {
          let dot = self.lex_ctx.peek_md_nth(1, SKIP_NONE);
          if dot.token.kind() == SyntaxKind::MdSymbol && dot.token.text().collect::<String>() == "."
          {
            break;
          }
        }
        continue;
      }

      let (block, early_exit) = self.parse_md_block_element();
      children.push(block);
      if early_exit.is_some_and(|ctx| ctx != ExprCtx::MdOrderedListItem) {
        self.expr_ctx_stack.exit(ExprCtx::MdOrderedListItem);
        return (
          self.emit(SyntaxKind::OrderedListItem, &children),
          early_exit,
        );
      }
      if early_exit == Some(ExprCtx::MdOrderedListItem) {
        break;
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::MdOrderedListItem);
    (self.emit(SyntaxKind::OrderedListItem, &children), None)
  }

  /// Parse a toggle list: `>- ...`.
  /// INVARIANT: Next tokens must be MdSymbol `>` followed by MdSymbol `-`.
  pub(in crate::parse) fn parse_toggle_list(&mut self) -> (GreenNode, Option<ExprCtx>) {
    debug_assert!(
      self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::MdSymbol
        && self
          .lex_ctx
          .peek_md(SKIP_NONE)
          .token
          .text()
          .collect::<String>()
          == ">"
        && self.lex_ctx.peek_md_nth(1, SKIP_NONE).token.kind() == SyntaxKind::MdSymbol
        && self
          .lex_ctx
          .peek_md_nth(1, SKIP_NONE)
          .token
          .text()
          .collect::<String>()
          == "-",
      "[ParseCtx::parse_toggle_list] Expected > followed by -"
    );

    let mut children = vec![];

    self.expr_ctx_stack.enter(ExprCtx::MdToggleList);

    // Parse first toggle item
    let (item, early_exit) = self.parse_toggle_list_item();
    children.push(item);
    if early_exit.is_some_and(|ctx| ctx != ExprCtx::MdToggleList) {
      self.expr_ctx_stack.exit(ExprCtx::MdToggleList);
      return (self.emit(SyntaxKind::ToggleList, &children), early_exit);
    }

    // Parse remaining toggle items
    loop {
      if !self.consume_md_newline_and_prefix(&mut children) {
        break;
      }
      let next = self.lex_ctx.peek_md(SKIP_NONE);
      let next_next = self.lex_ctx.peek_md_nth(1, SKIP_NONE);
      if next.token.kind() != SyntaxKind::MdSymbol
        || next.token.text().collect::<String>() != ">"
        || next_next.token.kind() != SyntaxKind::MdSymbol
        || next_next.token.text().collect::<String>() != "-"
      {
        break;
      }

      let (item, early_exit) = self.parse_toggle_list_item();
      children.push(item);
      if early_exit.is_some_and(|ctx| ctx != ExprCtx::MdToggleList) {
        self.expr_ctx_stack.exit(ExprCtx::MdToggleList);
        return (self.emit(SyntaxKind::ToggleList, &children), early_exit);
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::MdToggleList);
    (self.emit(SyntaxKind::ToggleList, &children), None)
  }

  /// Parse a toggle list item: `>- summary\n\n   details`.
  /// INVARIANT: Next token must be MdSymbol `>-`.
  fn parse_toggle_list_item(&mut self) -> (GreenNode, Option<ExprCtx>) {
    debug_assert!(
      self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::MdSymbol
        && self
          .lex_ctx
          .peek_md(SKIP_NONE)
          .token
          .text()
          .collect::<String>()
          == ">"
        && self.lex_ctx.peek_md_nth(1, SKIP_NONE).token.kind() == SyntaxKind::MdSymbol
        && self
          .lex_ctx
          .peek_md_nth(1, SKIP_NONE)
          .token
          .text()
          .collect::<String>()
          == "-",
      "[ParseCtx::parse_toggle_list_item] Expected > followed by -"
    );

    let mut children = vec![];

    self.expr_ctx_stack.enter(ExprCtx::MdToggleListItem);

    // Consume `>` and `-`
    self.advance_md(&mut children, SKIP_NONE);
    self.advance_md(&mut children, SKIP_NONE);

    // Require a space after `>-`
    if self.lex_ctx.peek_md(SKIP_NONE).token.kind() != SyntaxKind::Whitespace {
      self.emit_diagnostic(Diagnostic::MissingRequiredSpacesBetweenHashAndHeading {
        start_offset: self.offset(),
        end_offset: self.offset(),
      });
    } else {
      self.advance_md(&mut children, SKIP_NONE);
    }

    // Parse summary: inline elements on this line
    let mut summary_children = vec![];
    loop {
      let next_kind = self.lex_ctx.peek_md(SKIP_NONE).token.kind();
      if matches!(next_kind, SyntaxKind::Newline | SyntaxKind::Eof) {
        break;
      }
      let (inline, early_exit) = self.parse_md_inline_element();
      summary_children.push(inline);
      if early_exit.is_some() {
        children.push(self.emit(SyntaxKind::ToggleListSummary, &summary_children));
        self.expr_ctx_stack.exit(ExprCtx::MdToggleListItem);
        return (self.emit(SyntaxKind::ToggleListItem, &children), early_exit);
      }
    }
    children.push(self.emit(SyntaxKind::ToggleListSummary, &summary_children));

    // Check for blank line separating summary from details
    let next_kind = self.lex_ctx.peek_md(SKIP_NONE).token.kind();
    if next_kind == SyntaxKind::Eof {
      self.expr_ctx_stack.exit(ExprCtx::MdToggleListItem);
      return (self.emit(SyntaxKind::ToggleListItem, &children), None);
    }

    // Consume the newline after summary
    self.advance_md(&mut children, SKIP_NONE);

    // Check for blank line
    let next_kind = self.lex_ctx.peek_md(SKIP_NONE).token.kind();
    if next_kind != SyntaxKind::Newline {
      // No blank line: no details section
      self.expr_ctx_stack.exit(ExprCtx::MdToggleListItem);
      return (self.emit(SyntaxKind::ToggleListItem, &children), None);
    }

    // Consume the blank line
    self.advance_md(&mut children, SKIP_NONE);

    // Parse details: block elements until end of toggle item
    let mut details_children = vec![];
    loop {
      if !self.consume_md_newline_and_prefix(&mut children) {
        break;
      }
      let next = self.lex_ctx.peek_md(SKIP_NONE);
      if matches!(next.token.kind(), SyntaxKind::Newline | SyntaxKind::Eof) {
        break;
      }

      let (block, early_exit) = self.parse_md_block_element();
      details_children.push(block);
      if early_exit.is_some_and(|ctx| ctx != ExprCtx::MdToggleListItem) {
        children.push(self.emit(SyntaxKind::ToggleListDetails, &details_children));
        self.expr_ctx_stack.exit(ExprCtx::MdToggleListItem);
        return (self.emit(SyntaxKind::ToggleListItem, &children), early_exit);
      }
      if early_exit == Some(ExprCtx::MdToggleListItem) {
        break;
      }
    }
    if !details_children.is_empty() {
      children.push(self.emit(SyntaxKind::ToggleListDetails, &details_children));
    }

    self.expr_ctx_stack.exit(ExprCtx::MdToggleListItem);
    (self.emit(SyntaxKind::ToggleListItem, &children), None)
  }

  /// Parse a callout block: `::: label ... :::`.
  /// INVARIANT: Expect ::: to be the next token, all spaces must already be consumed and passed
  pub(in crate::parse) fn parse_callout_block(&mut self) -> (GreenNode, Option<ExprCtx>) {
    debug_assert!(
      self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::MdSymbol
        && self
          .lex_ctx
          .peek_md(SKIP_NONE)
          .token
          .text()
          .collect::<String>()
          == ":::",
      "[ParseCtx::parse_callout_block] Expected :::"
    );

    let mut children = vec![];
    let open_offset = self.offset();

    // Consume `:::`
    self.advance_md(&mut children, SKIP_NONE);

    // Require a space between `:::` and the label
    let next = self.lex_ctx.peek_md(SKIP_NONE);
    if next.token.kind() != SyntaxKind::Whitespace {
      self.emit_diagnostic(Diagnostic::MissingRequiredSpacesBetweenHashAndHeading {
        start_offset: self.offset(),
        end_offset: self.offset(),
      });
    } else {
      self.advance_md(&mut children, SKIP_NONE);
    }

    // Require a label identifier
    if self.lex_ctx.peek_md(SKIP_NONE).token.kind() != SyntaxKind::Ident {
      self.emit_diagnostic(Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Ident,
        start_offset: self.offset(),
        end_offset: self.offset(),
      });
    } else {
      self.advance_md(&mut children, SKIP_NONE);
    }

    // Consume the newline after the label (skip trailing whitespace)
    self.consume_md(
      &mut children,
      SKIP_WS,
      SyntaxKind::Newline,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Newline,
        start_offset: self.offset(),
        end_offset: self.offset(),
      },
    );

    let parent_prefix_count = self.expr_ctx_stack.md_prefix_tokens().len();
    let callout_ctx = ExprCtx::MdCalloutBlock(parent_prefix_count as u16);
    self.expr_ctx_stack.enter(callout_ctx);

    // Parse block elements until closing `:::` or EOF
    // The callout creates a new indentation context: inner elements start at indent 0
    loop {
      let next_kind = self.lex_ctx.peek_md(SKIP_NONE).token.kind();
      if next_kind == SyntaxKind::Eof {
        break;
      }

      // Check for closing `:::` at the same indentation level as the opening
      // The closing `:::` should appear right after the parent prefix
      let closing_pos = parent_prefix_count;
      let next = self.lex_ctx.peek_md_nth(closing_pos, SKIP_NONE);
      if next.token.kind() == SyntaxKind::MdSymbol && next.token.text().collect::<String>() == ":::"
      {
        break;
      }

      let (block, early_exit) = self.parse_md_block_element();
      children.push(block);
      if early_exit.is_some_and(|ctx| !ctx.is_md_callout_block()) {
        self.expr_ctx_stack.exit(callout_ctx);
        return (self.emit(SyntaxKind::CalloutBlock, &children), early_exit);
      }
      if early_exit.is_some_and(|ctx| ctx.is_md_callout_block()) {
        if let Some(ctx) = self.synchronize_callout_block(&mut children) {
          self.expr_ctx_stack.exit(callout_ctx);
          return (self.emit(SyntaxKind::CalloutBlock, &children), Some(ctx));
        }
      }
    }

    // Consume closing `:::`
    self.consume_md_if(
      &mut children,
      SKIP_WS,
      |token| token.kind() == SyntaxKind::MdSymbol && token.text().collect::<String>() == ":::",
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::CalloutBlock,
        start_offset: open_offset,
        end_offset: self.offset(),
      },
    );

    self.expr_ctx_stack.exit(callout_ctx);
    (self.emit(SyntaxKind::CalloutBlock, &children), None)
  }

  // Stop on `:::` at matching indent, or EOF.
  fn synchronize_callout_block(&mut self, children: &mut Vec<GreenNode>) -> Option<ExprCtx> {
    let current = self.expr_ctx_stack.current().unwrap();
    let mut error_children = vec![];
    let result = loop {
      let peek = self.lex_ctx.peek_md(SKIP_WS);
      let is_closing =
        peek.token.kind() == SyntaxKind::MdSymbol && peek.token.text().collect::<String>() == ":::";
      if is_closing || peek.token.kind() == SyntaxKind::Eof {
        break None;
      }
      if let Some(ctx) = self.consume_or_delegate_md(current, &mut error_children) {
        break Some(ctx);
      }
    };
    if !error_children.is_empty() {
      children.push(self.emit(SyntaxKind::Error, &error_children));
    }
    result
  }

  /// Parse a link: `[text](url)`.
  /// INVARIANT: The next token must be LBracket.
  pub(in crate::parse) fn parse_link(&mut self) -> (GreenNode, Option<ExprCtx>) {
    debug_assert!(
      self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::LBracket,
      "[ParseCtx::parse_link] Expected ["
    );

    let mut children = vec![];
    let open_offset = self.offset();

    // Consume `[`
    let ok = self.consume_md(
      &mut children,
      SKIP_NONE,
      SyntaxKind::LBracket,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Link,
        start_offset: open_offset,
        end_offset: open_offset,
      },
    );
    if !ok {
      let handler = self
        .expr_ctx_stack
        .find_handler(&self.lex_ctx.peek_md(SKIP_NONE).token);
      return (self.emit(SyntaxKind::Link, &children), handler);
    }

    self.expr_ctx_stack.enter(ExprCtx::MdLinkText);

    // Consume inline elements until `]` or end of inline element
    loop {
      if self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::RBracket {
        break;
      }
      if self.should_end_inline_element(&mut children) {
        self.emit_diagnostic(Diagnostic::UnclosedLink {
          start_offset: open_offset,
          end_offset: self.offset(),
        });
        self.expr_ctx_stack.exit(ExprCtx::MdLinkText);
        return (self.emit(SyntaxKind::Link, &children), None);
      }

      let (inline, early_exit) = self.parse_md_inline_element();
      children.push(inline);

      if early_exit.is_some_and(|ctx| ctx != ExprCtx::MdLinkText) {
        self.expr_ctx_stack.exit(ExprCtx::MdLinkText);
        return (self.emit(SyntaxKind::Link, &children), early_exit);
      }

      if early_exit == Some(ExprCtx::MdLinkText) {
        if let Some(ctx) = self.synchronize_link_text(&mut children) {
          self.expr_ctx_stack.exit(ExprCtx::MdLinkText);
          return (self.emit(SyntaxKind::Link, &children), Some(ctx));
        }
      }
    }

    // Consume `]`
    let ok = self.consume_md(
      &mut children,
      SKIP_NONE,
      SyntaxKind::RBracket,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Link,
        start_offset: open_offset,
        end_offset: open_offset,
      },
    );
    self.expr_ctx_stack.exit(ExprCtx::MdLinkText);
    if !ok {
      let handler = self
        .expr_ctx_stack
        .find_handler(&self.lex_ctx.peek_md(SKIP_NONE).token);
      return (self.emit(SyntaxKind::Link, &children), handler);
    }

    // Consume `(`
    let ok = self.consume_md(
      &mut children,
      SKIP_NONE,
      SyntaxKind::LParen,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Link,
        start_offset: open_offset,
        end_offset: open_offset,
      },
    );
    if !ok {
      let handler = self
        .expr_ctx_stack
        .find_handler(&self.lex_ctx.peek_md(SKIP_NONE).token);
      return (self.emit(SyntaxKind::Link, &children), handler);
    }

    self.expr_ctx_stack.enter(ExprCtx::MdLinkUrl);

    // Consume plain text tokens until `)`, Newline, or EOF
    let mut url_children = vec![];
    loop {
      let peek = self.lex_ctx.peek_md(SKIP_NONE);
      match peek.token.kind() {
        SyntaxKind::RParen | SyntaxKind::Newline | SyntaxKind::Eof => break,
        _ => {
          if let Some(ctx) = self.consume_or_delegate_md(ExprCtx::MdLinkUrl, &mut url_children) {
            children.push(self.emit(SyntaxKind::Text, &url_children));
            self.expr_ctx_stack.exit(ExprCtx::MdLinkUrl);
            return (self.emit(SyntaxKind::Link, &children), Some(ctx));
          }
        }
      }
    }
    children.push(self.emit(SyntaxKind::Text, &url_children));

    // Consume `)`
    let ok = self.consume_md(
      &mut children,
      SKIP_NONE,
      SyntaxKind::RParen,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Link,
        start_offset: open_offset,
        end_offset: open_offset,
      },
    );
    self.expr_ctx_stack.exit(ExprCtx::MdLinkUrl);
    if !ok {
      let handler = self
        .expr_ctx_stack
        .find_handler(&self.lex_ctx.peek_md(SKIP_NONE).token);
      return (self.emit(SyntaxKind::Link, &children), handler);
    }

    (self.emit(SyntaxKind::Link, &children), None)
  }

  /// Parse a media embed: `![alt](src)`.
  /// INVARIANT: The next token must be MdSymbol `!` followed by `[`.
  pub(in crate::parse) fn parse_media(&mut self) -> (GreenNode, Option<ExprCtx>) {
    debug_assert!(
      self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::MdSymbol
        && self
          .lex_ctx
          .peek_md(SKIP_NONE)
          .token
          .text()
          .collect::<String>()
          == "!",
      "[ParseCtx::parse_media] Expected !"
    );
    debug_assert!(
      self.lex_ctx.peek_md_nth(1, SKIP_NONE).token.kind() == SyntaxKind::LBracket,
      "[ParseCtx::parse_media] Expected [ after !"
    );

    let mut children = vec![];
    let open_offset = self.offset();

    // Consume `!`
    let ok = self.consume_md_if(
      &mut children,
      SKIP_NONE,
      |token| token.kind() == SyntaxKind::MdSymbol && token.text().collect::<String>() == "!",
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Media,
        start_offset: open_offset,
        end_offset: open_offset,
      },
    );
    if !ok {
      let handler = self
        .expr_ctx_stack
        .find_handler(&self.lex_ctx.peek_md(SKIP_NONE).token);
      return (self.emit(SyntaxKind::Media, &children), handler);
    }

    // Consume `[`
    let ok = self.consume_md(
      &mut children,
      SKIP_NONE,
      SyntaxKind::LBracket,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Media,
        start_offset: open_offset,
        end_offset: open_offset,
      },
    );
    if !ok {
      let handler = self
        .expr_ctx_stack
        .find_handler(&self.lex_ctx.peek_md(SKIP_NONE).token);
      return (self.emit(SyntaxKind::Media, &children), handler);
    }

    self.expr_ctx_stack.enter(ExprCtx::MdLinkText);

    // Consume inline elements until `]` or end of inline element
    loop {
      if self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::RBracket {
        break;
      }
      if self.should_end_inline_element(&mut children) {
        self.expr_ctx_stack.exit(ExprCtx::MdLinkText);
        return (self.emit(SyntaxKind::Media, &children), None);
      }

      let (inline, early_exit) = self.parse_md_inline_element();
      children.push(inline);

      if early_exit.is_some_and(|ctx| ctx != ExprCtx::MdLinkText) {
        self.expr_ctx_stack.exit(ExprCtx::MdLinkText);
        return (self.emit(SyntaxKind::Media, &children), early_exit);
      }

      if early_exit == Some(ExprCtx::MdLinkText) {
        if let Some(ctx) = self.synchronize_link_text(&mut children) {
          self.expr_ctx_stack.exit(ExprCtx::MdLinkText);
          return (self.emit(SyntaxKind::Media, &children), Some(ctx));
        }
      }
    }

    // Consume `]`
    let ok = self.consume_md(
      &mut children,
      SKIP_NONE,
      SyntaxKind::RBracket,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Media,
        start_offset: open_offset,
        end_offset: open_offset,
      },
    );
    self.expr_ctx_stack.exit(ExprCtx::MdLinkText);
    if !ok {
      let handler = self
        .expr_ctx_stack
        .find_handler(&self.lex_ctx.peek_md(SKIP_NONE).token);
      return (self.emit(SyntaxKind::Media, &children), handler);
    }

    // Consume `(`
    let ok = self.consume_md(
      &mut children,
      SKIP_NONE,
      SyntaxKind::LParen,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Media,
        start_offset: open_offset,
        end_offset: open_offset,
      },
    );
    if !ok {
      let handler = self
        .expr_ctx_stack
        .find_handler(&self.lex_ctx.peek_md(SKIP_NONE).token);
      return (self.emit(SyntaxKind::Media, &children), handler);
    }

    self.expr_ctx_stack.enter(ExprCtx::MdLinkUrl);

    // Consume plain text tokens until `)`, Newline, or EOF
    let mut url_children = vec![];
    loop {
      let peek = self.lex_ctx.peek_md(SKIP_NONE);
      match peek.token.kind() {
        SyntaxKind::RParen | SyntaxKind::Newline | SyntaxKind::Eof => break,
        _ => {
          if let Some(ctx) = self.consume_or_delegate_md(ExprCtx::MdLinkUrl, &mut url_children) {
            children.push(self.emit(SyntaxKind::Text, &url_children));
            self.expr_ctx_stack.exit(ExprCtx::MdLinkUrl);
            return (self.emit(SyntaxKind::Media, &children), Some(ctx));
          }
        }
      }
    }
    children.push(self.emit(SyntaxKind::Text, &url_children));

    // Consume `)`
    let ok = self.consume_md(
      &mut children,
      SKIP_NONE,
      SyntaxKind::RParen,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Media,
        start_offset: open_offset,
        end_offset: open_offset,
      },
    );
    self.expr_ctx_stack.exit(ExprCtx::MdLinkUrl);
    if !ok {
      let handler = self
        .expr_ctx_stack
        .find_handler(&self.lex_ctx.peek_md(SKIP_NONE).token);
      return (self.emit(SyntaxKind::Media, &children), handler);
    }

    (self.emit(SyntaxKind::Media, &children), None)
  }

  // Stop on `]`, Newline, EOF, or end of inline element.
  fn synchronize_link_text(&mut self, children: &mut Vec<GreenNode>) -> Option<ExprCtx> {
    let mut error_children = vec![];
    let result = loop {
      let peek = self.lex_ctx.peek_md(SKIP_NONE);
      if matches!(
        peek.token.kind(),
        SyntaxKind::RBracket | SyntaxKind::Newline | SyntaxKind::Eof
      ) || self.should_end_inline_element(children)
      {
        break None;
      }
      if let Some(ctx) = self.consume_or_delegate_md(ExprCtx::MdLinkText, &mut error_children) {
        break Some(ctx);
      }
    };
    if !error_children.is_empty() {
      children.push(self.emit(SyntaxKind::Error, &error_children));
    }
    result
  }

  /// Parse a footnote reference: `[^key]`.
  pub(in crate::parse) fn parse_footnote_ref(&mut self) -> (GreenNode, Option<ExprCtx>) {
    debug_assert!(
      self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::LBracket,
      "[ParseCtx::parse_footnote_ref] Expected ["
    );
    debug_assert!(
      {
        let second = self.lex_ctx.peek_md_nth(1, SKIP_NONE);
        second.token.kind() == SyntaxKind::MdSymbol
          && second.token.text().collect::<String>() == "^"
      },
      "[ParseCtx::parse_footnote_ref] Expected ^ after ["
    );

    let mut children = vec![];
    let open_offset = self.offset();

    self.expr_ctx_stack.enter(ExprCtx::MdCitation);
    self.advance_md(&mut children, SKIP_NONE); // consume `[`

    self.consume_md_if(
      &mut children,
      SKIP_NONE,
      |token| token.kind() == SyntaxKind::MdSymbol && token.text().collect::<String>() == "^",
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Citation,
        start_offset: open_offset,
        end_offset: open_offset,
      },
    );

    self.consume_md(
      &mut children,
      SKIP_NONE,
      SyntaxKind::Ident,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Citation,
        start_offset: open_offset,
        end_offset: open_offset,
      },
    );

    self.consume_md(
      &mut children,
      SKIP_NONE,
      SyntaxKind::RBracket,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Citation,
        start_offset: open_offset,
        end_offset: open_offset,
      },
    );

    self.expr_ctx_stack.exit(ExprCtx::MdCitation);
    (self.emit(SyntaxKind::Citation, &children), None)
  }

  /// Parse a citation: `[@key]`.
  /// INVARIANT: The next token must be LBracket.
  pub(in crate::parse) fn parse_citation(&mut self) -> (GreenNode, Option<ExprCtx>) {
    debug_assert!(
      self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::LBracket,
      "[ParseCtx::parse_citation] Expected ["
    );

    let mut children = vec![];
    let open_offset = self.offset();

    self.expr_ctx_stack.enter(ExprCtx::MdCitation);
    self.advance_md(&mut children, SKIP_NONE); // consume `[`

    self.consume_md_if(
      &mut children,
      SKIP_NONE,
      |token| token.kind() == SyntaxKind::MdSymbol && token.text().collect::<String>() == "@",
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Citation,
        start_offset: open_offset,
        end_offset: open_offset,
      },
    );

    self.consume_md(
      &mut children,
      SKIP_NONE,
      SyntaxKind::Ident,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Citation,
        start_offset: open_offset,
        end_offset: open_offset,
      },
    );

    self.consume_md(
      &mut children,
      SKIP_NONE,
      SyntaxKind::RBracket,
      Diagnostic::MissingSyntaxNode {
        expected: SyntaxKind::Citation,
        start_offset: open_offset,
        end_offset: open_offset,
      },
    );

    self.expr_ctx_stack.exit(ExprCtx::MdCitation);
    (self.emit(SyntaxKind::Citation, &children), None)
  }

  /// Parse bold text: `**text**`.
  /// INVARIANT: The next token must be MdSymbol `**`.
  /// Leading whitespace must already be consumed by the caller.
  /// Trailing whitespace after the closing delimiter is not consumed.
  pub(in crate::parse) fn parse_bold(&mut self) -> (GreenNode, Option<ExprCtx>) {
    debug_assert!(
      self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::MdSymbol
        && self
          .lex_ctx
          .peek_md(SKIP_NONE)
          .token
          .text()
          .collect::<String>()
          == "**",
      "[ParseCtx::parse_bold] Expected opening **"
    );

    let mut children = vec![];
    let open_offset = self.offset();

    self.expr_ctx_stack.enter(ExprCtx::MdBold);
    self.advance_md(&mut children, SKIP_NONE);

    loop {
      let text: String = self.lex_ctx.peek_md(SKIP_NONE).token.text().collect();
      if self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::MdSymbol && text == "**" {
        self.advance_md(&mut children, SKIP_NONE);
        break;
      }
      if self.should_end_inline_element(&mut children) {
        self.emit_diagnostic(Diagnostic::UnclosedBold {
          start_offset: open_offset,
          end_offset: self.offset(),
        });
        break;
      }
      if self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::Newline {
        self.advance_md(&mut children, SKIP_NONE);
        continue;
      }
      let (inline, early_exit) = self.parse_md_inline_element();
      children.push(inline);
      if early_exit.is_some_and(|ctx| ctx != ExprCtx::MdBold) {
        self.expr_ctx_stack.exit(ExprCtx::MdBold);
        return (self.emit(SyntaxKind::Bold, &children), early_exit);
      }
      if early_exit == Some(ExprCtx::MdBold) {
        if let Some(ctx) = self.synchronize_bold(&mut children) {
          self.expr_ctx_stack.exit(ExprCtx::MdBold);
          return (self.emit(SyntaxKind::Bold, &children), Some(ctx));
        }
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::MdBold);
    (self.emit(SyntaxKind::Bold, &children), None)
  }

  // Stop on `**`, EOF, or end of inline element.
  fn synchronize_bold(&mut self, children: &mut Vec<GreenNode>) -> Option<ExprCtx> {
    let mut error_children = vec![];
    let result = loop {
      let peek = self.lex_ctx.peek_md(SKIP_NONE);
      let is_closing =
        peek.token.kind() == SyntaxKind::MdSymbol && peek.token.text().collect::<String>() == "**";
      if is_closing
        || peek.token.kind() == SyntaxKind::Eof
        || self.should_end_inline_element(children)
      {
        break None;
      }
      if let Some(ctx) = self.consume_or_delegate_md(ExprCtx::MdBold, &mut error_children) {
        break Some(ctx);
      }
    };
    if !error_children.is_empty() {
      children.push(self.emit(SyntaxKind::Error, &error_children));
    }
    result
  }

  /// Parse italic text: `*text*` or `_text_`.
  /// INVARIANT: The next token must be MdSymbol `*` or `_`.
  /// Leading whitespace must already be consumed by the caller.
  /// Trailing whitespace after the closing delimiter is not consumed.
  pub(in crate::parse) fn parse_italic(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let opening: String = self.lex_ctx.peek_md(SKIP_NONE).token.text().collect();
    debug_assert!(
      self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::MdSymbol
        && (opening == "*" || opening == "_"),
      "[ParseCtx::parse_italic] Expected opening * or _"
    );

    let ctx = if opening == "*" {
      ExprCtx::MdItalicStar
    } else {
      ExprCtx::MdItalicUnderscore
    };
    let mut children = vec![];
    let open_offset = self.offset();

    self.expr_ctx_stack.enter(ctx);
    self.advance_md(&mut children, SKIP_NONE);

    loop {
      let text: String = self.lex_ctx.peek_md(SKIP_NONE).token.text().collect();
      if self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::MdSymbol
        && (text == "*" || text == "_")
      {
        self.advance_md(&mut children, SKIP_NONE);
        if text != opening {
          self.emit_diagnostic(Diagnostic::MismatchedItalicDelimiter {
            start_offset: open_offset,
            end_offset: self.offset(),
          });
        }
        break;
      }
      if self.should_end_inline_element(&mut children) {
        self.emit_diagnostic(Diagnostic::UnclosedItalic {
          start_offset: open_offset,
          end_offset: self.offset(),
        });
        break;
      }
      if self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::Newline {
        self.advance_md(&mut children, SKIP_NONE);
        continue;
      }
      let (inline, early_exit) = self.parse_md_inline_element();
      children.push(inline);
      if early_exit.is_some_and(|c| c != ctx) {
        self.expr_ctx_stack.exit(ctx);
        return (self.emit(SyntaxKind::Italic, &children), early_exit);
      }
      if early_exit == Some(ctx) {
        if let Some(propagate) = self.synchronize_italic(&opening, &mut children) {
          self.expr_ctx_stack.exit(ctx);
          return (self.emit(SyntaxKind::Italic, &children), Some(propagate));
        }
      }
    }

    self.expr_ctx_stack.exit(ctx);
    (self.emit(SyntaxKind::Italic, &children), None)
  }

  // Stop on `*`/`_` matching `opening`, EOF, or end of inline element.
  fn synchronize_italic(
    &mut self,
    opening: &str,
    children: &mut Vec<GreenNode>,
  ) -> Option<ExprCtx> {
    let ctx = if opening == "*" {
      ExprCtx::MdItalicStar
    } else {
      ExprCtx::MdItalicUnderscore
    };
    let mut error_children = vec![];
    let result = loop {
      let peek = self.lex_ctx.peek_md(SKIP_NONE);
      let text: String = peek.token.text().collect();
      let is_closing = peek.token.kind() == SyntaxKind::MdSymbol && (text == "*" || text == "_");
      if is_closing
        || peek.token.kind() == SyntaxKind::Eof
        || self.should_end_inline_element(children)
      {
        break None;
      }
      if let Some(propagate) = self.consume_or_delegate_md(ctx, &mut error_children) {
        break Some(propagate);
      }
    };
    if !error_children.is_empty() {
      children.push(self.emit(SyntaxKind::Error, &error_children));
    }
    result
  }

  /// Parse bolditalic text: `***text***`.
  /// INVARIANT: The next token must be MdSymbol `***`.
  /// Leading whitespace must already be consumed by the caller.
  /// Trailing whitespace after the closing delimiter is not consumed.
  pub(in crate::parse) fn parse_bold_italic(&mut self) -> (GreenNode, Option<ExprCtx>) {
    debug_assert!(
      self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::MdSymbol
        && self
          .lex_ctx
          .peek_md(SKIP_NONE)
          .token
          .text()
          .collect::<String>()
          == "***",
      "[ParseCtx::parse_bold_italic] Expected opening ***"
    );

    let mut children = vec![];
    let open_offset = self.offset();

    self.expr_ctx_stack.enter(ExprCtx::MdBoldItalic);
    self.advance_md(&mut children, SKIP_NONE);

    loop {
      let text: String = self.lex_ctx.peek_md(SKIP_NONE).token.text().collect();
      if self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::MdSymbol && text == "***" {
        self.advance_md(&mut children, SKIP_NONE);
        break;
      }
      if self.should_end_inline_element(&mut children) {
        self.emit_diagnostic(Diagnostic::UnclosedBoldItalic {
          start_offset: open_offset,
          end_offset: self.offset(),
        });
        break;
      }
      if self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::Newline {
        self.advance_md(&mut children, SKIP_NONE);
        continue;
      }
      let (inline, early_exit) = self.parse_md_inline_element();
      children.push(inline);
      if early_exit.is_some_and(|ctx| ctx != ExprCtx::MdBoldItalic) {
        self.expr_ctx_stack.exit(ExprCtx::MdBoldItalic);
        return (self.emit(SyntaxKind::BoldItalic, &children), early_exit);
      }
      if early_exit == Some(ExprCtx::MdBoldItalic) {
        if let Some(ctx) = self.synchronize_bold_italic(&mut children) {
          self.expr_ctx_stack.exit(ExprCtx::MdBoldItalic);
          return (self.emit(SyntaxKind::BoldItalic, &children), Some(ctx));
        }
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::MdBoldItalic);
    (self.emit(SyntaxKind::BoldItalic, &children), None)
  }

  // Stop on `***`, EOF, or end of inline element.
  fn synchronize_bold_italic(&mut self, children: &mut Vec<GreenNode>) -> Option<ExprCtx> {
    let mut error_children = vec![];
    let result = loop {
      let peek = self.lex_ctx.peek_md(SKIP_NONE);
      let is_closing =
        peek.token.kind() == SyntaxKind::MdSymbol && peek.token.text().collect::<String>() == "***";
      if is_closing
        || peek.token.kind() == SyntaxKind::Eof
        || self.should_end_inline_element(children)
      {
        break None;
      }
      if let Some(ctx) = self.consume_or_delegate_md(ExprCtx::MdBoldItalic, &mut error_children) {
        break Some(ctx);
      }
    };
    if !error_children.is_empty() {
      children.push(self.emit(SyntaxKind::Error, &error_children));
    }
    result
  }

  /// Parse strikethrough text: `~~text~~`.
  /// INVARIANT: The next token must be MdSymbol `~~`.
  /// Leading whitespace must already be consumed by the caller.
  /// Trailing whitespace after the closing delimiter is not consumed.
  pub(in crate::parse) fn parse_strikethrough(&mut self) -> (GreenNode, Option<ExprCtx>) {
    debug_assert!(
      self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::MdSymbol
        && self
          .lex_ctx
          .peek_md(SKIP_NONE)
          .token
          .text()
          .collect::<String>()
          == "~~",
      "[ParseCtx::parse_strikethrough] Expected opening ~~"
    );

    let mut children = vec![];
    let open_offset = self.offset();

    self.expr_ctx_stack.enter(ExprCtx::MdStrikethrough);
    self.advance_md(&mut children, SKIP_NONE);

    loop {
      let text: String = self.lex_ctx.peek_md(SKIP_NONE).token.text().collect();
      if self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::MdSymbol && text == "~~" {
        self.advance_md(&mut children, SKIP_NONE);
        break;
      }
      if self.should_end_inline_element(&mut children) {
        self.emit_diagnostic(Diagnostic::UnclosedStrikethrough {
          start_offset: open_offset,
          end_offset: self.offset(),
        });
        break;
      }
      if self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::Newline {
        self.advance_md(&mut children, SKIP_NONE);
        continue;
      }
      let (inline, early_exit) = self.parse_md_inline_element();
      children.push(inline);
      if early_exit.is_some_and(|ctx| ctx != ExprCtx::MdStrikethrough) {
        self.expr_ctx_stack.exit(ExprCtx::MdStrikethrough);
        return (self.emit(SyntaxKind::Strikethrough, &children), early_exit);
      }
      if early_exit == Some(ExprCtx::MdStrikethrough) {
        if let Some(ctx) = self.synchronize_strikethrough(&mut children) {
          self.expr_ctx_stack.exit(ExprCtx::MdStrikethrough);
          return (self.emit(SyntaxKind::Strikethrough, &children), Some(ctx));
        }
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::MdStrikethrough);
    (self.emit(SyntaxKind::Strikethrough, &children), None)
  }

  // Stop on `~~`, EOF, or end of inline element.
  fn synchronize_strikethrough(&mut self, children: &mut Vec<GreenNode>) -> Option<ExprCtx> {
    let mut error_children = vec![];
    let result = loop {
      let peek = self.lex_ctx.peek_md(SKIP_NONE);
      let is_closing =
        peek.token.kind() == SyntaxKind::MdSymbol && peek.token.text().collect::<String>() == "~~";
      if is_closing
        || peek.token.kind() == SyntaxKind::Eof
        || self.should_end_inline_element(children)
      {
        break None;
      }
      if let Some(ctx) = self.consume_or_delegate_md(ExprCtx::MdStrikethrough, &mut error_children)
      {
        break Some(ctx);
      }
    };
    if !error_children.is_empty() {
      children.push(self.emit(SyntaxKind::Error, &error_children));
    }
    result
  }

  /// Parse a text run: consecutive plain text, including surrounding whitespace.
  /// Consumes leading and trailing spaces.
  pub(in crate::parse) fn parse_text(&mut self) -> (GreenNode, Option<ExprCtx>) {
    let mut children = vec![];

    loop {
      let next_kind = self.lex_ctx.peek_md(SKIP_NONE).token.kind();
      if matches!(next_kind, SyntaxKind::Newline | SyntaxKind::Eof) {
        break;
      }
      if self.is_md_inline_start() {
        break;
      }
      self.advance_md(&mut children, SKIP_NONE);
    }

    (self.emit(SyntaxKind::Text, &children), None)
  }

  /// Consume a newline and the expected prefix on the next line.
  fn consume_md_newline_and_prefix(&mut self, children: &mut Vec<GreenNode>) -> bool {
    // Consume trailing whitespace and the newline
    self.advance_md(children, SKIP_WS);

    // Consume tokens matching the expected prefix tokens
    let expected_tokens: Vec<SyntaxToken> = self.expr_ctx_stack.md_prefix_tokens().to_vec();
    for expected_token in &expected_tokens {
      let peek = self.lex_ctx.peek_md(SKIP_NONE);
      if peek.token != *expected_token {
        self.emit_diagnostic(Diagnostic::MissingExpectMdPrefix {
          expected_prefix: format!("{:?}", expected_tokens),
          start_offset: self.offset(),
          end_offset: self.offset(),
        });
        return false;
      }
      self.advance_md(children, SKIP_NONE);
    }

    true
  }

  // If the next token should be handled by an outer context, return that context.
  // Otherwise consume the token into `error_children` for the caller to wrap as Error.
  fn consume_or_delegate_md(
    &mut self,
    current: ExprCtx,
    error_children: &mut Vec<GreenNode>,
  ) -> Option<ExprCtx> {
    let handler = self
      .expr_ctx_stack
      .find_handler(&self.lex_ctx.peek_md(SKIP_NONE).token);
    if handler.is_some_and(|ctx| ctx != current) {
      return handler;
    }
    self.advance_md(error_children, SKIP_NONE);
    None
  }

  /// Whether the current inline element should end due to EOF or a line boundary.
  /// Consumes the newline and prefix if present.
  fn should_end_inline_element(&mut self, children: &mut Vec<GreenNode>) -> bool {
    let next_kind = self.lex_ctx.peek_md(SKIP_NONE).token.kind();
    if next_kind == SyntaxKind::Eof {
      return true;
    }
    if next_kind == SyntaxKind::Newline {
      self.consume_md_newline_and_prefix(children);
      let after = self.lex_ctx.peek_md(SKIP_NONE);
      if matches!(after.token.kind(), SyntaxKind::Newline | SyntaxKind::Eof) {
        return true;
      }
      if !self.peek_md_prefix() {
        return true;
      }
      if after.token.kind() == SyntaxKind::MdSymbol {
        let text: String = after.token.text().collect();
        let first = text.chars().next().unwrap_or('\0');
        if matches!(first, '#' | '-' | '*' | '+' | '>' | '|' | ':') {
          return true;
        }
      }
      if after.token.kind() == SyntaxKind::MdNumber {
        return true;
      }
      if matches!(
        after.token.kind(),
        SyntaxKind::CodeBlock | SyntaxKind::MathBlock
      ) {
        return true;
      }
    }
    false
  }

  /// Whether the next token starts an inline element.
  fn is_md_inline_start(&mut self) -> bool {
    let next = self.lex_ctx.peek_md(SKIP_NONE);
    match next.token.kind() {
      SyntaxKind::LBracket => true,
      SyntaxKind::InterpStart => true,
      SyntaxKind::InlineMath | SyntaxKind::InlineCode => true,
      SyntaxKind::MdSymbol => {
        let text: String = next.token.text().collect();
        if matches!(text.as_str(), "*" | "_" | "**" | "***" | "~~") {
          return true;
        }
        // `![` starts a media embed
        if text == "!" {
          let second = self.lex_ctx.peek_md_nth(1, SKIP_NONE);
          return second.token.kind() == SyntaxKind::LBracket;
        }
        false
      }
      _ => false,
    }
  }

  /// Peek whether the next token is a a newline & is followed by the expected prefix.
  /// Does not consume anything.
  /// INVARIANT: The next token must be a Newline.
  fn peek_md_newline_and_prefix(&mut self) -> bool {
    debug_assert!(
      self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::Newline,
      "[ParseCtx::peek_md_newline_and_prefix] Expected next token to be Newline"
    );
    let expected_tokens: Vec<SyntaxToken> = self.expr_ctx_stack.md_prefix_tokens().to_vec();
    for (idx, expected_token) in expected_tokens.iter().enumerate() {
      let peek = self.lex_ctx.peek_md_nth(idx + 1, SKIP_NONE);
      if peek.token != *expected_token {
        return false;
      }
    }
    true
  }

  /// Peek and check if upcoming tokens match the expected prefix.
  /// INVARIANT: Must be called after consuming a newline.
  fn peek_md_prefix(&mut self) -> bool {
    let expected_tokens: Vec<SyntaxToken> = self.expr_ctx_stack.md_prefix_tokens().to_vec();
    for (idx, expected_token) in expected_tokens.iter().enumerate() {
      let peek = self.lex_ctx.peek_md_nth(idx, SKIP_NONE);
      if peek.token != *expected_token {
        return false;
      }
    }
    true
  }

  /// Whether the next non-leading-whitespace token starts a block-level element.
  /// INVARIANT: Must be called right after consuming a newline
  fn is_md_block_start(&mut self) -> bool {
    if !self.peek_md_prefix() {
      return false;
    }

    self.is_md_any_block_start(SKIP_NONE)
  }
}

// Block element start detection helpers
impl<S: Utf8Stream> ParseCtx<S> {
  fn is_heading_start(&mut self, skip: u16) -> bool {
    let next = self.lex_ctx.peek_md(skip);
    next.token.kind() == SyntaxKind::MdSymbol && next.token.text().all(|c| c == '#')
  }

  fn is_bullet_list_start(&mut self, skip: u16) -> bool {
    let next = self.lex_ctx.peek_md(skip);
    if next.token.kind() != SyntaxKind::MdSymbol {
      return false;
    }
    let text: String = next.token.text().collect();
    matches!(text.as_str(), "-" | "*" | "+")
      && self.lex_ctx.peek_md_nth(1, skip).token.kind() == SyntaxKind::Whitespace
  }

  fn is_ordered_list_start(&mut self, skip: u16) -> bool {
    let next = self.lex_ctx.peek_md(skip);
    if next.token.kind() != SyntaxKind::MdNumber {
      return false;
    }
    let dot = self.lex_ctx.peek_md_nth(1, skip);
    dot.token.kind() == SyntaxKind::MdSymbol && dot.token.text().collect::<String>() == "."
  }

  fn is_blockquote_start(&mut self, skip: u16) -> bool {
    let next = self.lex_ctx.peek_md(skip);
    if next.token.kind() != SyntaxKind::MdSymbol {
      return false;
    }
    let text: String = next.token.text().collect();
    text == ">"
      && !(self.lex_ctx.peek_md_nth(1, skip).token.kind() == SyntaxKind::MdSymbol
        && self
          .lex_ctx
          .peek_md_nth(1, skip)
          .token
          .text()
          .collect::<String>()
          == "-")
  }

  fn is_toggle_list_start(&mut self, skip: u16) -> bool {
    let next = self.lex_ctx.peek_md(skip);
    if next.token.kind() != SyntaxKind::MdSymbol {
      return false;
    }
    let text: String = next.token.text().collect();
    text == ">"
      && self.lex_ctx.peek_md_nth(1, skip).token.kind() == SyntaxKind::MdSymbol
      && self
        .lex_ctx
        .peek_md_nth(1, skip)
        .token
        .text()
        .collect::<String>()
        == "-"
  }

  fn is_table_start(&mut self, skip: u16) -> bool {
    let next = self.lex_ctx.peek_md(skip);
    next.token.kind() == SyntaxKind::MdSymbol && next.token.text().collect::<String>() == "|"
  }

  fn is_callout_start(&mut self, skip: u16) -> bool {
    let next = self.lex_ctx.peek_md(skip);
    next.token.kind() == SyntaxKind::MdSymbol && next.token.text().collect::<String>() == ":::"
  }

  fn is_media_block_start(&mut self, skip: u16) -> bool {
    let next = self.lex_ctx.peek_md(skip);
    if next.token.kind() != SyntaxKind::MdSymbol {
      return false;
    }
    next.token.text().collect::<String>() == "!"
      && self.lex_ctx.peek_md_nth(1, skip).token.kind() == SyntaxKind::LBracket
  }

  fn is_code_or_math_block_start(&mut self, skip: u16) -> bool {
    let next = self.lex_ctx.peek_md(skip);
    matches!(
      next.token.kind(),
      SyntaxKind::CodeBlock | SyntaxKind::MathBlock
    )
  }

  fn is_md_any_block_start(&mut self, skip: u16) -> bool {
    self.is_heading_start(skip)
      || self.is_bullet_list_start(skip)
      || self.is_ordered_list_start(skip)
      || self.is_blockquote_start(skip)
      || self.is_toggle_list_start(skip)
      || self.is_table_start(skip)
      || self.is_callout_start(skip)
      || self.is_media_block_start(skip)
      || self.is_code_or_math_block_start(skip)
  }
}
