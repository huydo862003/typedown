/// A lexer error.
/// When multiple variants match, use the first (most specific) one.
pub enum LexDiagnostic {
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
}
