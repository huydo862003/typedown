pub(super) const SKIP_NONE: u8 = 0;
pub(super) const SKIP_WS: u8 = 1 << 0; // skip whitespace
pub(super) const SKIP_COMMENT: u8 = 1 << 1; // skip comments
pub(super) const SKIP_NEWLINE: u8 = 1 << 2; // skip newlines
pub(super) const SKIP_WC: u8 = SKIP_WS | SKIP_COMMENT;
pub(super) const SKIP_WCN: u8 = SKIP_WS | SKIP_COMMENT | SKIP_NEWLINE;
