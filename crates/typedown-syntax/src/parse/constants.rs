pub(super) const SKIP_NONE: u16 = 0;

pub(super) const SKIP_LEADING_WS: u16 = 1 << 0; // whitespace at line start (before content)
pub(super) const SKIP_TRAILING_WS: u16 = 1 << 1; // whitespace at line end (after content, before newline)
pub(super) const SKIP_MIDDLE_WS: u16 = 1 << 2; // whitespace between tokens on the same line
pub(super) const SKIP_STANDALONE_WS: u16 = 1 << 3; // whitespace on an otherwise empty line

pub(super) const SKIP_WS: u16 =
  SKIP_LEADING_WS | SKIP_TRAILING_WS | SKIP_MIDDLE_WS | SKIP_STANDALONE_WS;
pub(super) const SKIP_EMPTY_LINE: u16 = SKIP_STANDALONE_WS | SKIP_NEWLINE;

pub(super) const SKIP_COMMENT: u16 = 1 << 4; // skip comments
pub(super) const SKIP_NEWLINE: u16 = 1 << 5; // skip newlines
pub(super) const SKIP_INDENT: u16 = 1 << 6; // skip YamlIndent
pub(super) const SKIP_DEDENT: u16 = 1 << 7; // skip YamlDedent

pub(super) const SKIP_WC: u16 = SKIP_WS | SKIP_COMMENT;
pub(super) const SKIP_WCN: u16 = SKIP_WS | SKIP_COMMENT | SKIP_NEWLINE;
