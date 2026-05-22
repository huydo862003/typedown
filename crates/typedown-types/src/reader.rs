/// Character-level input source.
/// Abstracts over files, strings, stdin, etc.
pub trait Reader {
  /// Look at the next character without consuming it.
  /// Returns None at EOF.
  fn peek(&mut self) -> Option<char>;

  /// Consume and return the next character.
  /// Returns None at EOF.
  fn advance(&mut self) -> Option<char>;

  /// Current byte offset in the source.
  fn offset(&self) -> usize;
}
