//! Pretty-printer for the markdown body of a TDR file
//!
//! Rules (some inspired from Google's style guide):
//! - Exactly one space after `#` in headings
//! - Exactly one blank line before/after headings
//! - No trailing whitespace
//! - Collapse multiple consecutive blank lines to one
//! - 2-space indent for nested list content
//! - Code blocks pass through verbatim
//! - Paragraph content passes through verbatim (no reflowing)
//! - File ends with exactly one newline

use crate::syntax::ast::{
  AstNode, MdBlockElement, MdBody, MdBulletList, MdBulletListItem, MdOrderedList,
  MdOrderedListItem, MdTaskListItem,
};
use crate::syntax::red::RedNode;
use crate::syntax::syntax_kind::SyntaxKind;

/// Format the AST markdown body of a TDR file
pub fn format_markdown(body: &MdBody) -> String {
  let mut out = String::from("\n");
  let blocks: Vec<_> = body.block_elements().collect();

  for (idx, block) in blocks.iter().enumerate() {
    let is_first = idx == 0;
    let prev_was_heading = idx > 0 && is_heading(&blocks[idx - 1]);
    let is_heading_block = is_heading(block);

    // Blank line before heading (unless first block or previous was also a heading)
    if is_heading_block && !is_first && !prev_was_heading {
      ensure_blank_line(&mut out);
    }

    format_block(&mut out, block.syntax(), 0);

    // Blank line after heading
    if is_heading_block {
      ensure_blank_line(&mut out);
    }
  }

  // Collapse multiple blank lines and ensure trailing newline
  let result = collapse_blank_lines(&out);
  ensure_trailing_newline(result)
}

fn is_heading(block: &MdBlockElement) -> bool {
  block.syntax().kind() == SyntaxKind::MdHeading
}

fn format_block(out: &mut String, node: &RedNode, depth: usize) {
  match node.kind() {
    SyntaxKind::MdHeading => format_heading(out, node),
    SyntaxKind::MdBulletList => format_bullet_list(out, node, depth),
    SyntaxKind::MdOrderedList => format_ordered_list(out, node, depth),
    _ => {
      // Tables, blockquotes, callouts, paragraphs: emit source text
      emit_source_lines(out, node, depth);
    }
  }
}

/// Format a heading: normalize to exactly one space after `#` symbols
fn format_heading(out: &mut String, node: &RedNode) {
  let text = node.text();
  let trimmed = text.trim();

  // Count leading `#` symbols
  let hash_count = trimmed.chars().take_while(|ch| *ch == '#').count();
  if hash_count == 0 {
    // Not a valid heading, emit as-is
    push_trimmed_line(out, trimmed);
    return;
  }

  // Extract heading content after the hashes
  let after_hashes = &trimmed[hash_count..];
  let content = after_hashes.trim_start();

  if content.is_empty() {
    push_trimmed_line(out, &"#".repeat(hash_count));
  } else {
    push_trimmed_line(out, &format!("{} {}", "#".repeat(hash_count), content));
  }
}

/// Format a bullet list, applying 2-space indentation per nesting level
fn format_bullet_list(out: &mut String, node: &RedNode, depth: usize) {
  if let Some(list) = MdBulletList::cast(node.clone()) {
    for item in list.items() {
      format_bullet_item(out, &item, depth);
    }
  }
  // Also handle task list items at this level
  for child in node.children() {
    if child.kind() == SyntaxKind::MdTaskListItem
      && let Some(task) = MdTaskListItem::cast(child)
    {
      format_task_item(out, &task, depth);
    }
  }
}

/// Format a single bullet list item
fn format_bullet_item(out: &mut String, item: &MdBulletListItem, depth: usize) {
  let indent = "  ".repeat(depth);
  let source = item.syntax().text();
  let first_line = source.lines().next().unwrap_or("");

  // Normalize the first line: indent + "- " + content
  let content = first_line
    .trim()
    .strip_prefix("- ")
    .or_else(|| first_line.trim().strip_prefix("* "))
    .or_else(|| first_line.trim().strip_prefix("+ "))
    .unwrap_or(first_line.trim());
  push_trimmed_line(out, &format!("{indent}- {content}"));
}

/// Format a task list item: `- [ ] text` or `- [x] text`
fn format_task_item(out: &mut String, item: &MdTaskListItem, depth: usize) {
  let indent = "  ".repeat(depth);
  let source = item.syntax().text();
  let first_line = source.lines().next().unwrap_or("");
  let trimmed = first_line.trim();

  push_trimmed_line(out, &format!("{indent}{trimmed}"));
}

/// Format an ordered list
fn format_ordered_list(out: &mut String, node: &RedNode, depth: usize) {
  if let Some(list) = MdOrderedList::cast(node.clone()) {
    for (idx, item) in list.items().enumerate() {
      format_ordered_item(out, &item, depth, idx + 1);
    }
  }
}

/// Format a single ordered list item
fn format_ordered_item(out: &mut String, item: &MdOrderedListItem, depth: usize, number: usize) {
  let indent = "  ".repeat(depth);
  let source = item.syntax().text();
  let first_line = source.lines().next().unwrap_or("");

  // Strip existing number and dot, emit with correct number
  let trimmed = first_line.trim();
  let content = trimmed
    .find(". ")
    .map(|pos| &trimmed[pos + 2..])
    .unwrap_or(trimmed);
  push_trimmed_line(out, &format!("{indent}{number}. {content}"));
}

/// Emit source text lines with trailing whitespace stripped, at the given indent depth
fn emit_source_lines(out: &mut String, node: &RedNode, depth: usize) {
  let indent = "  ".repeat(depth);
  let text = node.text();
  for line in text.lines() {
    if depth == 0 {
      push_trimmed_line(out, line);
    } else {
      let trimmed = line.trim();
      if trimmed.is_empty() {
        out.push('\n');
      } else {
        push_trimmed_line(out, &format!("{indent}{trimmed}"));
      }
    }
  }
}

/// Push a line with trailing whitespace removed
fn push_trimmed_line(out: &mut String, line: &str) {
  out.push_str(line.trim_end());
  out.push('\n');
}

/// Ensure the output ends with exactly one blank line (two newlines)
fn ensure_blank_line(out: &mut String) {
  if !out.ends_with("\n\n") {
    if out.ends_with('\n') {
      out.push('\n');
    } else {
      out.push_str("\n\n");
    }
  }
}

/// Collapse runs of 3+ consecutive newlines into exactly 2 (one blank line)
fn collapse_blank_lines(text: &str) -> String {
  let mut result = String::with_capacity(text.len());
  let mut consecutive_newlines = 0;

  for ch in text.chars() {
    if ch == '\n' {
      consecutive_newlines += 1;
      if consecutive_newlines <= 2 {
        result.push(ch);
      }
    } else {
      consecutive_newlines = 0;
      result.push(ch);
    }
  }

  result
}

/// Ensure the text ends with exactly one newline
fn ensure_trailing_newline(mut text: String) -> String {
  while text.ends_with("\n\n") {
    text.pop();
  }
  if !text.ends_with('\n') {
    text.push('\n');
  }
  text
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::syntax::ast::SourceFile;
  use crate::syntax::parse::tests::helpers::parse;

  fn fmt(source: &str) -> String {
    let (green, _) = parse(source);
    let red = RedNode::from_green(0, green);
    let file = SourceFile::cast(red).expect("should parse as SourceFile");
    let body = file.body().expect("should have a body");
    format_markdown(&body)
  }

  // Heading gets exactly one space after hashes
  #[test]
  fn heading_spacing() {
    let result = fmt("---\n---\n##Heading\n");
    assert_eq!(result, "\n## Heading\n");
  }

  // Extra spaces after hashes are normalized
  #[test]
  fn heading_extra_spaces() {
    let result = fmt("---\n---\n##   Heading\n");
    assert_eq!(result, "\n## Heading\n");
  }

  // Blank line before heading
  #[test]
  fn blank_line_before_heading() {
    let result = fmt("---\n---\nSome text.\n## Heading\n");
    assert!(
      result.contains("\n\n## Heading"),
      "should have blank line before heading:\n{result}"
    );
  }

  // Blank line after heading
  #[test]
  fn blank_line_after_heading() {
    let result = fmt("---\n---\n## Heading\nSome text.\n");
    assert!(
      result.contains("## Heading\n\n"),
      "should have blank line after heading:\n{result}"
    );
  }

  // Trailing whitespace removed
  #[test]
  fn trailing_whitespace() {
    let result = fmt("---\n---\nHello world   \n");
    assert_eq!(result, "\nHello world\n");
  }

  // Multiple blank lines collapsed to one
  #[test]
  fn collapse_blank_lines_test() {
    let result = fmt("---\n---\nFirst.\n\n\n\nSecond.\n");
    assert_eq!(result, "\nFirst.\n\nSecond.\n");
  }

  // File ends with exactly one newline
  #[test]
  fn trailing_newline() {
    let result = fmt("---\n---\nHello\n\n\n");
    assert!(
      result.ends_with("\nHello\n"),
      "should end with one newline:\n{result:?}"
    );
  }

  // Formatter is idempotent
  #[test]
  fn idempotent() {
    let input = "---\n---\n##Heading\n\n\n\nSome text.   \n\n## Another\n\nParagraph.\n";
    let first = fmt(input);
    let second_input = format!("---\n---\n{first}");
    let second = fmt(&second_input);
    assert_eq!(first, second, "formatter should be idempotent");
  }

  // Simple bullet list
  #[test]
  fn bullet_list() {
    let result = fmt(
      r#"---
---
- First item
- Second item
- Third item
"#,
    );
    assert_eq!(
      result,
      r#"
- First item
- Second item
- Third item
"#
    );
  }

  // Mixed bullet prefixes normalized to -
  #[test]
  fn mixed_bullet_prefixes() {
    let result = fmt(
      r#"---
---
* Star item
+ Plus item
- Dash item
"#,
    );
    assert_eq!(
      result,
      r#"
- Star item
- Plus item
- Dash item
"#
    );
  }

  // List followed by heading
  #[test]
  fn list_then_heading() {
    let result = fmt(
      r#"---
---
- Item one
- Item two
## Next Section
"#,
    );
    assert!(
      result.contains("- Item two\n\n## Next Section"),
      "should have blank line between list and heading:\n{result}"
    );
  }

  // Heading followed by list
  #[test]
  fn heading_then_list() {
    let result = fmt(
      r#"---
---
## Section

- Item one
- Item two
"#,
    );
    assert_eq!(
      result,
      r#"
## Section

- Item one
- Item two
"#
    );
  }

  // Full document with mixed elements
  #[test]
  fn full_document() {
    let result = fmt(
      r#"---
---

Alice is a **backend developer**.

## Skills

| Area | Proficiency |
|------|-------------|
| Rust | Expert |

## Responsibilities

- Lead backend development
- Review pull requests
- Mentor junior developers
"#,
    );
    assert!(
      result.contains("- Lead backend development\n"),
      "list items preserved:\n{result}"
    );
    assert!(
      result.contains("## Skills\n\n"),
      "blank line after heading:\n{result}"
    );
    assert!(
      !result.contains("- Lead backend development\n- Lead backend development"),
      "no duplication:\n{result}"
    );
  }

  // Idempotent with lists
  #[test]
  fn idempotent_with_lists() {
    let input = r#"---
---
## Section

- Item one
- Item two

Some text.
"#;
    let first = fmt(input);
    let second_input = format!("---\n---\n{first}");
    let second = fmt(&second_input);
    assert_eq!(first, second, "formatter should be idempotent with lists");
  }
}
