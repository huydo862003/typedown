use crate::stream::{Utf8Result, Utf8Stream};

/// A Utf8Stream over a borrowed string slice.
/// Since the input is already valid UTF-8, this never produces Utf8Result::Invalid.
pub struct StringStream<'a> {
  str: &'a str,
  offset: usize,
  buffer: Option<Utf8Result>,
}

impl<'a> StringStream<'a> {
  pub fn new(str: &'a str) -> Self {
    Self {
      str,
      offset: 0,
      buffer: None,
    }
  }
}

impl<'a> Utf8Stream for StringStream<'a> {
  fn peek(&mut self) -> Utf8Result {
    if let Some(result) = self.buffer {
      return result;
    }

    let result = match self.str[self.offset..].chars().next() {
      Some(char) => Utf8Result::Char(char),
      None => Utf8Result::Eof,
    };

    self.buffer = Some(result);
    result
  }

  fn advance(&mut self) -> Utf8Result {
    let result = match self.buffer.take() {
      Some(r) => r,
      None => {
        let r = self.peek();
        self.buffer.take();
        r
      }
    };

    if let Utf8Result::Char(char) = &result {
      self.offset += char.len_utf8();
    }

    result
  }

  fn offset(&self) -> usize {
    self.offset
  }

  fn exhausted(&self) -> bool {
    self.offset >= self.str.len()
  }
}
