pub(in crate::parse) const SKIP_NONE: u16 = 0;

pub(in crate::parse) const SKIP_LEADING_WS: u16 = 1 << 0; // whitespace at line start (before content)
pub(in crate::parse) const SKIP_TRAILING_WS: u16 = 1 << 1; // whitespace at line end (after content, before newline)
pub(in crate::parse) const SKIP_MIDDLE_WS: u16 = 1 << 2; // whitespace between tokens on the same line
pub(in crate::parse) const SKIP_STANDALONE_WS: u16 = 1 << 3; // whitespace on an otherwise empty line

pub(in crate::parse) const SKIP_WS: u16 =
  SKIP_LEADING_WS | SKIP_TRAILING_WS | SKIP_MIDDLE_WS | SKIP_STANDALONE_WS;
pub(in crate::parse) const SKIP_EMPTY_LINE: u16 = SKIP_STANDALONE_WS | SKIP_NEWLINE;

pub(in crate::parse) const SKIP_COMMENT: u16 = 1 << 4; // skip comments
pub(in crate::parse) const SKIP_NEWLINE: u16 = 1 << 5; // skip newlines
pub(in crate::parse) const SKIP_INDENT: u16 = 1 << 6; // skip YamlIndent
pub(in crate::parse) const SKIP_DEDENT: u16 = 1 << 7; // skip YamlDedent

pub(in crate::parse) const SKIP_INDENT_DEDENT: u16 = SKIP_INDENT | SKIP_DEDENT;
pub(in crate::parse) const SKIP_WC: u16 = SKIP_WS | SKIP_COMMENT;
pub(in crate::parse) const SKIP_WCN: u16 = SKIP_WS | SKIP_COMMENT | SKIP_NEWLINE;
pub(in crate::parse) const SKIP_ALL_TRIVIA: u16 = SKIP_WCN | SKIP_INDENT_DEDENT;
