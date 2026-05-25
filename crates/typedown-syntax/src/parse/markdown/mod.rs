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
    let mut children = vec![];
    self.expr_ctx_stack.enter(ExprCtx::MarkdownBody);

    todo!();

    self.expr_ctx_stack.exit(ExprCtx::MarkdownBody);
    self.emit(SyntaxKind::Body, &children)
  }

  pub(in crate::parse) fn parse_md_block_element(&mut self) -> (GreenNode, Option<ExprCtx>) {
    todo!()
  }

  pub(in crate::parse) fn parse_md_inline_element(&mut self) -> (GreenNode, Option<ExprCtx>) {
    todo!()
  }

  /// Parse a heading: `# ...`, `## ...`, etc.
  /// INVARIANT: The next token should be a hash sequence  /// Any whitespaces must be consumed by the parent to pass the correct current_indent
  pub(in crate::parse) fn parse_heading(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
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
        let (inline, early_exit) = self.parse_md_inline_element();
        children.push(inline);
        if early_exit.is_some() {
          return (self.emit(SyntaxKind::Heading, &children), early_exit);
        }
      }
    }

    // Consume the trailing newline if present
    let next_kind = self.lex_ctx.peek_md(SKIP_NONE).token.kind();
    if next_kind == SyntaxKind::Newline {
      self.advance_md(&mut children, SKIP_NONE);
    }

    (self.emit(SyntaxKind::Heading, &children), None)
  }

  /// Parse a paragraph: consecutive non-blank text lines.
  /// INVARIANT: The current line is not blank (caller must ensure there is content).
  pub(in crate::parse) fn parse_paragraph(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
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
          let (block, early_exit) = self.parse_md_block_element();
          children.push(block);
          if early_exit.is_some() {
            return (self.emit(SyntaxKind::Paragraph, &children), early_exit);
          }
        } else {
          // Block at same or lower indent: end paragraph
          break;
        }
      }
    }

    (self.emit(SyntaxKind::Paragraph, &children), None)
  }

  /// Parse a blockquote: `> ...`.
  pub(in crate::parse) fn parse_blockquote(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
    todo!()
  }

  /// Parse a table: `| ... | ... |`.
  pub(in crate::parse) fn parse_table(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
    todo!()
  }

  /// Parse a table row.
  pub(in crate::parse) fn parse_table_row(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
    todo!()
  }

  /// Parse a table cell.
  pub(in crate::parse) fn parse_table_cell(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
    todo!()
  }

  /// Parse a bullet list: `- ...` or `* ...`.
  pub(in crate::parse) fn parse_bullet_list(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
    todo!()
  }

  /// Parse a bullet list item.
  pub(in crate::parse) fn parse_bullet_list_item(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
    todo!()
  }

  /// Parse an ordered list: `1. ...`.
  pub(in crate::parse) fn parse_ordered_list(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
    todo!()
  }

  /// Parse an ordered list item.
  pub(in crate::parse) fn parse_ordered_list_item(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
    todo!()
  }

  /// Parse a toggle list: `>- ...`.
  pub(in crate::parse) fn parse_toggle_list(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
    todo!()
  }

  /// Parse a toggle list item.
  pub(in crate::parse) fn parse_toggle_list_item(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
    todo!()
  }

  /// Parse a callout block: `::: label ... :::`.
  pub(in crate::parse) fn parse_callout_block(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
    todo!()
  }

  /// Parse a footnote block: `:::footnote ... :::`.
  pub(in crate::parse) fn parse_footnote_block(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
    todo!()
  }

  /// Parse a bibliography block: `:::bibtex ... :::`.
  pub(in crate::parse) fn parse_bibliography_block(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
    todo!()
  }

  /// Parse a link: `[text](url)`.
  pub(in crate::parse) fn parse_link(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
    todo!()
  }

  /// Parse a media embed: `![alt](src)`.
  /// INVARIANT: The next token must be MdSymbol `!` followed by `[`.
  pub(in crate::parse) fn parse_media(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
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
      self.lex_ctx.peek_md_nth(2, SKIP_NONE).token.kind() == SyntaxKind::LBracket,
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
      if self.should_end_inline_element(current_indent) {
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
        if let Some(ctx) = self.synchronize_link_text(current_indent, &mut children) {
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
  fn synchronize_link_text(
    &mut self,
    current_indent: usize,
    children: &mut Vec<GreenNode>,
  ) -> Option<ExprCtx> {
    let mut error_children = vec![];
    let result = loop {
      let peek = self.lex_ctx.peek_md(SKIP_NONE);
      if matches!(
        peek.token.kind(),
        SyntaxKind::RBracket | SyntaxKind::Newline | SyntaxKind::Eof
      ) || self.should_end_inline_element(current_indent)
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
  pub(in crate::parse) fn parse_footnote_ref(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
    debug_assert!(
      self.lex_ctx.peek_md(SKIP_NONE).token.kind() == SyntaxKind::LBracket,
      "[ParseCtx::parse_footnote_ref] Expected ["
    );
    debug_assert!(
      {
        let second = self.lex_ctx.peek_md_nth(2, SKIP_NONE);
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
  pub(in crate::parse) fn parse_citation(
    &mut self,
    _current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
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
  pub(in crate::parse) fn parse_bold(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
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
      if self.should_end_inline_element(current_indent) {
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
        if let Some(ctx) = self.synchronize_bold(current_indent, &mut children) {
          self.expr_ctx_stack.exit(ExprCtx::MdBold);
          return (self.emit(SyntaxKind::Bold, &children), Some(ctx));
        }
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::MdBold);
    (self.emit(SyntaxKind::Bold, &children), None)
  }

  // Stop on `**`, EOF, or end of inline element.
  fn synchronize_bold(
    &mut self,
    current_indent: usize,
    children: &mut Vec<GreenNode>,
  ) -> Option<ExprCtx> {
    let mut error_children = vec![];
    let result = loop {
      let peek = self.lex_ctx.peek_md(SKIP_NONE);
      let is_closing =
        peek.token.kind() == SyntaxKind::MdSymbol && peek.token.text().collect::<String>() == "**";
      if is_closing
        || peek.token.kind() == SyntaxKind::Eof
        || self.should_end_inline_element(current_indent)
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
  pub(in crate::parse) fn parse_italic(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
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
      if self.should_end_inline_element(current_indent) {
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
        if let Some(propagate) = self.synchronize_italic(current_indent, &opening, &mut children) {
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
    current_indent: usize,
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
        || self.should_end_inline_element(current_indent)
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
  pub(in crate::parse) fn parse_bold_italic(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
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
      if self.should_end_inline_element(current_indent) {
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
        if let Some(ctx) = self.synchronize_bold_italic(current_indent, &mut children) {
          self.expr_ctx_stack.exit(ExprCtx::MdBoldItalic);
          return (self.emit(SyntaxKind::BoldItalic, &children), Some(ctx));
        }
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::MdBoldItalic);
    (self.emit(SyntaxKind::BoldItalic, &children), None)
  }

  // Stop on `***`, EOF, or end of inline element.
  fn synchronize_bold_italic(
    &mut self,
    current_indent: usize,
    children: &mut Vec<GreenNode>,
  ) -> Option<ExprCtx> {
    let mut error_children = vec![];
    let result = loop {
      let peek = self.lex_ctx.peek_md(SKIP_NONE);
      let is_closing =
        peek.token.kind() == SyntaxKind::MdSymbol && peek.token.text().collect::<String>() == "***";
      if is_closing
        || peek.token.kind() == SyntaxKind::Eof
        || self.should_end_inline_element(current_indent)
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
  pub(in crate::parse) fn parse_strikethrough(
    &mut self,
    current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
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
      if self.should_end_inline_element(current_indent) {
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
        if let Some(ctx) = self.synchronize_strikethrough(current_indent, &mut children) {
          self.expr_ctx_stack.exit(ExprCtx::MdStrikethrough);
          return (self.emit(SyntaxKind::Strikethrough, &children), Some(ctx));
        }
      }
    }

    self.expr_ctx_stack.exit(ExprCtx::MdStrikethrough);
    (self.emit(SyntaxKind::Strikethrough, &children), None)
  }

  // Stop on `~~`, EOF, or end of inline element.
  fn synchronize_strikethrough(
    &mut self,
    current_indent: usize,
    children: &mut Vec<GreenNode>,
  ) -> Option<ExprCtx> {
    let mut error_children = vec![];
    let result = loop {
      let peek = self.lex_ctx.peek_md(SKIP_NONE);
      let is_closing =
        peek.token.kind() == SyntaxKind::MdSymbol && peek.token.text().collect::<String>() == "~~";
      if is_closing
        || peek.token.kind() == SyntaxKind::Eof
        || self.should_end_inline_element(current_indent)
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
  pub(in crate::parse) fn parse_text(
    &mut self,
    _current_indent: usize,
  ) -> (GreenNode, Option<ExprCtx>) {
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
  /// Does not consume any tokens.
  fn should_end_inline_element(&mut self, current_indent: usize) -> bool {
    let next_kind = self.lex_ctx.peek_md(SKIP_NONE).token.kind();
    if next_kind == SyntaxKind::Eof {
      return true;
    }
    if next_kind == SyntaxKind::Newline {
      // Peek past the newline and any leading whitespace on the next line
      let after = self.lex_ctx.peek_md_nth(2, SKIP_LEADING_WS);
      if matches!(after.token.kind(), SyntaxKind::Newline | SyntaxKind::Eof) {
        return true; // blank line
      }
      if after.indent_depth <= current_indent && after.token.kind() == SyntaxKind::MdSymbol {
        let text: String = after.token.text().collect();
        let first = text.chars().next().unwrap_or('\0');
        if matches!(first, '#' | '-' | '*' | '+' | '>' | '|' | ':') {
          return true;
        }
      }
      if after.indent_depth <= current_indent && after.token.kind() == SyntaxKind::Number {
        return true; // ordered list item
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
      SyntaxKind::InlineMath
      | SyntaxKind::MathBlock
      | SyntaxKind::InlineCode
      | SyntaxKind::CodeBlock => true,
      SyntaxKind::MdSymbol => {
        let text: String = next.token.text().collect();
        if matches!(text.as_str(), "*" | "_" | "**" | "***" | "~~") {
          return true;
        }
        // `![` starts a media embed
        if text == "!" {
          let second = self.lex_ctx.peek_md_nth(2, SKIP_NONE);
          return second.token.kind() == SyntaxKind::LBracket;
        }
        false
      }
      _ => false,
    }
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

    // `![` starts a media embed block: requires the `!` to be followed by `[`
    if text == "!" {
      let second = self.lex_ctx.peek_md_nth(2, SKIP_LEADING_WS);
      return second.token.kind() == SyntaxKind::LBracket;
    }

    matches!(
      first,
      '#'  // heading
      | '-' | '*' | '+' // bullet list
      | '>' // blockquote or toggle list
      | '|' // table (may introduce false positives, but acceptable)
      | ':' // callout/footnote/bibliography (:::)
    )
  }
}
