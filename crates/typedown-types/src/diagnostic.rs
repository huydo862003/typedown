/// Compilation diagnostics.
/// When multiple variants match, use the first (most specific) one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Diagnostic {
  /* Lexer diagnostics */
  /// Expected a specific character but reached end of input.
  UnexpectedEof {
    expected: char,
    start_offset: usize,
    end_offset: usize,
  },

  /// Expected a specific character but found a different one.
  UnexpectedChar {
    expected: char,
    encountered: char,
    start_offset: usize,
    end_offset: usize,
  },

  /// A "..." or '...' string literal was opened but never closed.
  UnterminatedString {
    start_offset: usize,
    end_offset: usize,
  },

  /// A ${...} interpolation was opened but never closed.
  UnterminatedInterpolation {
    start_offset: usize,
    end_offset: usize,
  },

  /// A fenced code block (```) was opened but never closed.
  UnterminatedCodeBlock {
    start_offset: usize,
    end_offset: usize,
  },

  /// An inline code span (`) was opened but never closed.
  UnterminatedInlineCode {
    start_offset: usize,
    end_offset: usize,
  },

  /// A block math ($$) was opened but never closed.
  UnterminatedMathBlock {
    start_offset: usize,
    end_offset: usize,
  },

  /// An inline math ($) was opened but never closed.
  UnterminatedInlineMath {
    start_offset: usize,
    end_offset: usize,
  },

  /// A code block fence is missing a newline after the opening fence or before the closing fence.
  MissingCodeBlockNewline {
    start_offset: usize,
    end_offset: usize,
  },

  /// A math block delimiter is missing a newline after the opening $$ or before the closing $$.
  MissingMathBlockNewline {
    start_offset: usize,
    end_offset: usize,
  },

  /// Encountered a character that is not valid in the current lexing context.
  InvalidChar {
    encountered: char,
    start_offset: usize,
    end_offset: usize,
  },

  /// Encountered an invalid UTF-8 byte sequence.
  InvalidUtf8 {
    start_offset: usize,
    end_offset: usize,
  },

  /// Mixed tabs and spaces on the same indentation line.
  MixedIndentation {
    start_offset: usize,
    end_offset: usize,
  },

  /// Indentation uses a different character than what was established earlier.
  InconsistentIndentation {
    expected: char,
    encountered: char,
    start_offset: usize,
    end_offset: usize,
  },

  /// Dedent to an indentation level that was never established.
  UnmatchedDedent {
    indent: usize,
    start_offset: usize,
    end_offset: usize,
  },

  /// Missing digits after exponent in scientific notation (e.g. 2.5E+, 1e).
  MissingExponentDigits {
    start_offset: usize,
    end_offset: usize,
  },

  /* Parser diagnostics */
  /// Unexpected tokens found before/after the frontmatter marker ---
  UnexpectedTokensOnFrontmatterMarkerLine {
    start_offset: usize,
    end_offset: usize,
  },

  /// Missing frontmatter marker ---
  MissingFrontmatterMarker { offset: usize },

  /// Expected a specific syntax node or token but it was missing.
  MissingMarkdownHeadingHash {
    start_offset: usize,
    end_offset: usize,
  },

  /// Expected a specific syntax node or token but it was missing.
  MissingRequiredSpacesBetweenHashAndHeading {
    start_offset: usize,
    end_offset: usize,
  },

  /// Expected a specific syntax node or token but it was missing.
  MissingSyntaxNode {
    expected: crate::syntax_kind::SyntaxKind,
    start_offset: usize,
    end_offset: usize,
  },

  /// A link was opened with `[` but never closed with `](url)`.
  UnclosedLink {
    start_offset: usize,
    end_offset: usize,
  },

  /// A bold span was opened but never closed before a blank line, block boundary, or EOF.
  UnclosedBold {
    start_offset: usize,
    end_offset: usize,
  },

  /// An italic span was opened but never closed before a blank line, block boundary, or EOF.
  UnclosedItalic {
    start_offset: usize,
    end_offset: usize,
  },

  /// A strikethrough span was opened but never closed before a blank line, block boundary, or EOF.
  UnclosedStrikethrough {
    start_offset: usize,
    end_offset: usize,
  },

  /// A bolditalic span was opened but never closed before a blank line, block boundary, or EOF.
  UnclosedBoldItalic {
    start_offset: usize,
    end_offset: usize,
  },

  /// The closing delimiter of an italic span does not match the opening delimiter (`*` vs `_`).
  MismatchedItalicDelimiter {
    start_offset: usize,
    end_offset: usize,
  },

  /// The expected line prefix (e.g. `> ` for blockquotes) was not found after a newline.
  MissingExpectMdPrefix {
    expected_prefix: String,
    start_offset: usize,
    end_offset: usize,
  },

  /// A table is missing the required separator row after the header.
  MissingTableSeparatorRow {
    start_offset: usize,
    end_offset: usize,
  },

  /// A table row has a different number of columns than the header row.
  TableColumnCountMismatch {
    expected: usize,
    found: usize,
    start_offset: usize,
    end_offset: usize,
  },

  /// The indent of a block string content is not greater than the enclosing block indent.
  InsufficientBlockIndent {
    expected_more_than: usize,
    found: usize,
    start_offset: usize,
    end_offset: usize,
  },
}
