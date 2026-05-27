pub(in crate::parse) const SKIP_NONE: u16 = 0;

pub(in crate::parse) const SKIP_WS: u16 = 1 << 0;

pub(in crate::parse) const SKIP_COMMENT: u16 = 1 << 1;
pub(in crate::parse) const SKIP_NEWLINE: u16 = 1 << 2;
pub(in crate::parse) const SKIP_INDENT: u16 = 1 << 3;
pub(in crate::parse) const SKIP_DEDENT: u16 = 1 << 4;

pub(in crate::parse) const SKIP_INDENT_DEDENT: u16 = SKIP_INDENT | SKIP_DEDENT;
pub(in crate::parse) const SKIP_WC: u16 = SKIP_WS | SKIP_COMMENT;
pub(in crate::parse) const SKIP_WCN: u16 = SKIP_WS | SKIP_COMMENT | SKIP_NEWLINE;
pub(in crate::parse) const SKIP_ALL_TRIVIA: u16 = SKIP_WCN | SKIP_INDENT_DEDENT;
