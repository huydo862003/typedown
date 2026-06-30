/// A compact 8-byte type
pub struct Span {
  pub file_id: u32,
  pub start_offset: u16,
  pub end_offset: u16,
}
