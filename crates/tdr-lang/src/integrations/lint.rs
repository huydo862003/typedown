//! Markdown linter for TDR files
//!
//! Rules (some inspired from Google's style guide):
//! - Missing alt text on images
//! - Generic link text ("click here", "here", "link")
//! - Formatting violations (specific messages per violation)
//! - Multiple H1 headings
//! - Duplicate headings

use std::collections::HashMap;

use crate::syntax::ast::{AstNode, MdBody, MdHeading, MdLink, MdMedia};
use crate::syntax::red::RedNode;
use crate::syntax::syntax_kind::SyntaxKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintDiagnostic {
  pub start_offset: usize,
  pub end_offset: usize,
  pub code: LintCode,
  pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintCode {
  MissingAltText,
  GenericLinkText,
  MultipleH1,
  DuplicateHeading,
  TrailingWhitespace,
  HeadingSpacing,
  HeadingBlankLine,
  ConsecutiveBlankLines,
}

impl LintCode {
  pub fn as_str(self) -> &'static str {
    match self {
      LintCode::MissingAltText => "missing-alt-text",
      LintCode::GenericLinkText => "generic-link-text",
      LintCode::MultipleH1 => "multiple-h1",
      LintCode::DuplicateHeading => "duplicate-heading",
      LintCode::TrailingWhitespace => "trailing-whitespace",
      LintCode::HeadingSpacing => "heading-spacing",
      LintCode::HeadingBlankLine => "heading-blank-line",
      LintCode::ConsecutiveBlankLines => "consecutive-blank-lines",
    }
  }
}

const GENERIC_LINK_TEXTS: &[&str] = &[
  "click here",
  "here",
  "link",
  "this",
  "read more",
  "more",
  "this link",
];

/// Lint the markdown body of a TDR file.
pub fn lint_markdown(body: &MdBody) -> Vec<LintDiagnostic> {
  let mut diagnostics = Vec::new();

  lint_headings(body, &mut diagnostics);
  lint_inline_elements(body.syntax(), &mut diagnostics);
  lint_trailing_whitespace(body, &mut diagnostics);
  lint_heading_spacing(body, &mut diagnostics);
  lint_heading_blank_lines(body, &mut diagnostics);
  lint_consecutive_blank_lines(body, &mut diagnostics);

  diagnostics
}

/// Check for multiple H1 headings and duplicate heading text
fn lint_headings(body: &MdBody, diagnostics: &mut Vec<LintDiagnostic>) {
  let mut h1_count = 0;
  // heading text -> (level, first offset)
  let mut seen_headings: HashMap<String, usize> = HashMap::new();

  for block in body.block_elements() {
    if block.syntax().kind() != SyntaxKind::MdHeading {
      continue;
    }
    let Some(heading) = MdHeading::cast(block.syntax().clone()) else {
      continue;
    };

    let level = heading.level();
    let text = heading_text(&heading);
    let (offset, len) = heading.syntax().trimmed_range();

    // Multiple H1
    if level == 1 {
      h1_count += 1;
      if h1_count > 1 {
        diagnostics.push(LintDiagnostic {
          start_offset: offset,
          end_offset: offset + len,
          code: LintCode::MultipleH1,
          message: "Multiple H1 headings; a document should have only one".to_string(),
        });
      }
    }

    // Duplicate heading
    if let Some(&first_offset) = seen_headings.get(&text) {
      if first_offset != offset {
        diagnostics.push(LintDiagnostic {
          start_offset: offset,
          end_offset: offset + len,
          code: LintCode::DuplicateHeading,
          message: format!("Duplicate heading \"{}\"", text),
        });
      }
    } else {
      seen_headings.insert(text, offset);
    }
  }
}

/// Extract the text content of a heading (after the `#` symbols)
fn heading_text(heading: &MdHeading) -> String {
  let text = heading.syntax().text();
  let trimmed = text.trim();
  let hash_count = trimmed.chars().take_while(|ch| *ch == '#').count();
  trimmed[hash_count..].trim().to_string()
}

/// Walk the tree recursively to find links and images
fn lint_inline_elements(node: &RedNode, diagnostics: &mut Vec<LintDiagnostic>) {
  match node.kind() {
    SyntaxKind::MdMedia => {
      if let Some(media) = MdMedia::cast(node.clone()) {
        let alt = media.alt().map(|t| t.value()).unwrap_or_default();
        if alt.trim().is_empty() {
          let (offset, len) = node.trimmed_range();
          diagnostics.push(LintDiagnostic {
            start_offset: offset,
            end_offset: offset + len,
            code: LintCode::MissingAltText,
            message: "Image is missing alt text".to_string(),
          });
        }
      }
    }
    SyntaxKind::MdLink => {
      if let Some(link) = MdLink::cast(node.clone()) {
        let text = link.alt().map(|t| t.value()).unwrap_or_default();
        let lower = text.trim().to_lowercase();
        if GENERIC_LINK_TEXTS.contains(&lower.as_str()) {
          let (offset, len) = node.trimmed_range();
          diagnostics.push(LintDiagnostic {
            start_offset: offset,
            end_offset: offset + len,
            code: LintCode::GenericLinkText,
            message: format!(
              "Avoid generic link text \"{}\"; use a descriptive phrase",
              text.trim()
            ),
          });
        }
      }
    }
    _ => {}
  }

  for child in node.children() {
    lint_inline_elements(&child, diagnostics);
  }
}

/// Check for trailing whitespace on any line in the body
fn lint_trailing_whitespace(body: &MdBody, diagnostics: &mut Vec<LintDiagnostic>) {
  let source = body.syntax().text();
  let body_offset = body.syntax().offset();

  let mut offset = body_offset;
  for line in source.lines() {
    if line != line.trim_end() {
      diagnostics.push(LintDiagnostic {
        start_offset: offset,
        end_offset: offset + line.len(),
        code: LintCode::TrailingWhitespace,
        message: "Trailing whitespace".to_string(),
      });
    }
    offset += line.len() + 1; // +1 for newline
  }
}

/// Check that headings have exactly one space after `#` symbols
fn lint_heading_spacing(body: &MdBody, diagnostics: &mut Vec<LintDiagnostic>) {
  for block in body.block_elements() {
    if block.syntax().kind() != SyntaxKind::MdHeading {
      continue;
    }
    let text = block.syntax().text();
    let trimmed = text.trim();
    let hash_count = trimmed.chars().take_while(|ch| *ch == '#').count();
    if hash_count == 0 {
      continue;
    }
    let after_hashes = &trimmed[hash_count..];
    // Should be exactly one space, then content (or empty heading)
    if !after_hashes.is_empty() && !after_hashes.starts_with(' ') {
      let (offset, len) = block.syntax().trimmed_range();
      diagnostics.push(LintDiagnostic {
        start_offset: offset,
        end_offset: offset + len,
        code: LintCode::HeadingSpacing,
        message: "Missing space after # in heading".to_string(),
      });
    } else if after_hashes.starts_with("  ") {
      let (offset, len) = block.syntax().trimmed_range();
      diagnostics.push(LintDiagnostic {
        start_offset: offset,
        end_offset: offset + len,
        code: LintCode::HeadingSpacing,
        message: "Extra spaces after # in heading; use exactly one".to_string(),
      });
    }
  }
}

/// Check that headings have blank lines before and after them
fn lint_heading_blank_lines(body: &MdBody, diagnostics: &mut Vec<LintDiagnostic>) {
  let source = body.syntax().text();
  let body_offset = body.syntax().offset();

  for block in body.block_elements() {
    if block.syntax().kind() != SyntaxKind::MdHeading {
      continue;
    }
    let node = block.syntax();
    let (trimmed_offset, trimmed_len) = node.trimmed_range();
    // Convert absolute offsets to relative positions within body text
    let rel_start = node.offset().saturating_sub(body_offset);
    let rel_end = rel_start + node.text_len();

    // Check blank line before (unless at start of body)
    if rel_start > 0 {
      let before = &source[..rel_start];
      if !before.ends_with("\n\n") {
        diagnostics.push(LintDiagnostic {
          start_offset: trimmed_offset,
          end_offset: trimmed_offset + trimmed_len,
          code: LintCode::HeadingBlankLine,
          message: "Missing blank line before heading".to_string(),
        });
      }
    }

    // Check blank line after (unless at end of body)
    if rel_end < source.len() {
      let after = &source[rel_end..];
      if !after.starts_with("\n\n") && !after.is_empty() {
        diagnostics.push(LintDiagnostic {
          start_offset: trimmed_offset,
          end_offset: trimmed_offset + trimmed_len,
          code: LintCode::HeadingBlankLine,
          message: "Missing blank line after heading".to_string(),
        });
      }
    }
  }
}

/// Check for multiple consecutive blank lines
fn lint_consecutive_blank_lines(body: &MdBody, diagnostics: &mut Vec<LintDiagnostic>) {
  let source = body.syntax().text();
  let body_offset = body.syntax().offset();

  let mut consecutive_newlines = 0;
  for (idx, ch) in source.char_indices() {
    if ch == '\n' {
      consecutive_newlines += 1;
      if consecutive_newlines == 3 {
        diagnostics.push(LintDiagnostic {
          start_offset: body_offset + idx,
          end_offset: body_offset + idx + 1,
          code: LintCode::ConsecutiveBlankLines,
          message: "Multiple consecutive blank lines".to_string(),
        });
      }
    } else {
      consecutive_newlines = 0;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::syntax::ast::SourceFile;
  use crate::syntax::parse::tests::helpers::parse;
  use crate::syntax::red::RedNode;

  fn lint(source: &str) -> Vec<LintDiagnostic> {
    let (green, _) = parse(source);
    let red = RedNode::from_green(0, green);
    let file = SourceFile::cast(red).expect("should parse as SourceFile");
    let body = file.body().expect("should have a body");
    lint_markdown(&body)
  }

  fn codes(diags: &[LintDiagnostic]) -> Vec<&str> {
    diags.iter().map(|d| d.code.as_str()).collect()
  }

  // Missing alt text on image
  #[test]
  fn missing_alt_text() {
    let diags = lint("---\n---\n![](image.png)\n");
    assert!(
      codes(&diags).contains(&"missing-alt-text"),
      "should warn about missing alt text: {:?}",
      codes(&diags)
    );
  }

  // Image with alt text is fine
  #[test]
  fn image_with_alt_text() {
    let diags = lint("---\n---\n![A cat](cat.png)\n");
    assert!(
      !codes(&diags).contains(&"missing-alt-text"),
      "should not warn when alt text is present: {:?}",
      codes(&diags)
    );
  }

  // Generic link text
  #[test]
  fn generic_link_text() {
    let diags = lint("---\n---\n[click here](https://example.com)\n");
    assert!(
      codes(&diags).contains(&"generic-link-text"),
      "should warn about generic link text: {:?}",
      codes(&diags)
    );
  }

  // Descriptive link text is fine
  #[test]
  fn descriptive_link_text() {
    let diags = lint("---\n---\n[the full documentation](https://example.com)\n");
    assert!(
      !codes(&diags).contains(&"generic-link-text"),
      "should not warn on descriptive text: {:?}",
      codes(&diags)
    );
  }

  // Multiple H1 headings
  #[test]
  fn multiple_h1() {
    let diags = lint("---\n---\n# First\n\n# Second\n");
    assert!(
      codes(&diags).contains(&"multiple-h1"),
      "should warn about multiple H1: {:?}",
      codes(&diags)
    );
  }

  // Single H1 is fine
  #[test]
  fn single_h1() {
    let diags = lint("---\n---\n# Title\n\n## Section\n");
    assert!(
      !codes(&diags).contains(&"multiple-h1"),
      "should not warn on single H1: {:?}",
      codes(&diags)
    );
  }

  // Duplicate headings
  #[test]
  fn duplicate_headings() {
    let diags = lint("---\n---\n## Summary\n\nText.\n\n## Summary\n");
    assert!(
      codes(&diags).contains(&"duplicate-heading"),
      "should warn about duplicate headings: {:?}",
      codes(&diags)
    );
  }

  // Unique headings are fine
  #[test]
  fn unique_headings() {
    let diags = lint("---\n---\n## Overview\n\n## Details\n");
    assert!(
      !codes(&diags).contains(&"duplicate-heading"),
      "should not warn on unique headings: {:?}",
      codes(&diags)
    );
  }

  // Trailing whitespace
  #[test]
  fn trailing_whitespace() {
    let diags = lint("---\n---\nHello   \n");
    assert!(
      codes(&diags).contains(&"trailing-whitespace"),
      "should warn about trailing whitespace: {:?}",
      codes(&diags)
    );
  }

  // No trailing whitespace is fine
  #[test]
  fn no_trailing_whitespace() {
    let diags = lint("---\n---\nHello\n");
    assert!(
      !codes(&diags).contains(&"trailing-whitespace"),
      "should not warn without trailing whitespace: {:?}",
      codes(&diags)
    );
  }

  // Missing space after # in heading
  #[test]
  fn heading_missing_space() {
    let diags = lint("---\n---\n##Heading\n");
    assert!(
      codes(&diags).contains(&"heading-spacing"),
      "should warn about missing space: {:?}",
      codes(&diags)
    );
  }

  // Missing blank line before heading
  #[test]
  fn heading_missing_blank_line() {
    let diags = lint("---\n---\nSome text.\n## Heading\n");
    assert!(
      codes(&diags).contains(&"heading-blank-line"),
      "should warn about missing blank line: {:?}",
      codes(&diags)
    );
  }

  // Multiple consecutive blank lines
  #[test]
  fn consecutive_blank_lines() {
    let diags = lint("---\n---\nFirst.\n\n\n\nSecond.\n");
    assert!(
      codes(&diags).contains(&"consecutive-blank-lines"),
      "should warn about consecutive blank lines: {:?}",
      codes(&diags)
    );
  }

  // Well-formatted file has no warnings
  #[test]
  fn clean_file() {
    let diags = lint("---\n---\n## Heading\n\nSome text.\n");
    assert!(
      diags.is_empty(),
      "well-formatted file should have no lint warnings: {:?}",
      codes(&diags)
    );
  }
}
