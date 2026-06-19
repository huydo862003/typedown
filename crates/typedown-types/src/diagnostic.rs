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

  /* Vault config diagnostics */
  /// No typedown.yaml or typedown.yml found in the project root.
  MissingVaultConfig { root_dir: String },

  /// Failed to read the vault config file.
  VaultConfigReadError { path: String, message: String },

  /// Failed to parse the vault config file as YAML.
  VaultConfigParseError { path: String, message: String },

  /// The vault config file is empty.
  VaultConfigEmpty { path: String },

  /// A required field is missing from the vault config.
  VaultConfigMissingField { path: String, field: String },

  /* Typechecker diagnostics */
  /// Missing _type field in top-level mapping.
  MissingSchemaField {
    start_offset: usize,
    end_offset: usize,
  },

  /// Could not resolve _type reference.
  UnresolvedSchema {
    name: String,
    start_offset: usize,
    end_offset: usize,
  },

  /// Wrong number of type arguments passed to a type constructor.
  WrongTypeArgCount { expected: usize, got: usize },

  /// Callee expression is not callable.
  NotCallable {
    start_offset: usize,
    end_offset: usize,
  },

  /// Wrong number of arguments in a function call.
  WrongArgCount {
    expected: usize,
    got: usize,
    start_offset: usize,
    end_offset: usize,
  },

  /// Argument type does not match the expected parameter type.
  ArgTypeMismatch {
    expected: String,
    start_offset: usize,
    end_offset: usize,
  },

  /// A field value does not match the expected type declared by the schema.
  FieldTypeMismatch {
    field: String,
    expected: String,
    start_offset: usize,
    end_offset: usize,
  },

  /// Expression is not indexable.
  NotIndexable {
    start_offset: usize,
    end_offset: usize,
  },

  /// Index type does not match the container's key type.
  IndexTypeMismatch {
    expected: String,
    start_offset: usize,
    end_offset: usize,
  },

  /// Tag inner expression does not match the schema.
  TagTypeMismatch {
    expected: String,
    start_offset: usize,
    end_offset: usize,
  },

  /// Operand type does not match the expected type for the operator.
  OperandTypeMismatch {
    op: String,
    expected: String,
    start_offset: usize,
    end_offset: usize,
  },

  /// A required field declared by the schema is absent in the mapping.
  MissingRequiredField {
    field: String,
    start_offset: usize,
    end_offset: usize,
  },

  /// A sequence element does not match the declared element type.
  ElementTypeMismatch {
    expected: String,
    start_offset: usize,
    end_offset: usize,
  },

  /// A mapping has duplicate keys.
  DuplicateKey {
    key: String,
    start_offset: usize,
    end_offset: usize,
  },

  /// A file reference could not be resolved
  UnresolvedFileRef {
    path: String,
    start_offset: usize,
    end_offset: usize,
  },
}
