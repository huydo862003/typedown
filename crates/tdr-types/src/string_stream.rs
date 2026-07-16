use std::cell::Cell;

use crate::stream::{Utf8Result, Utf8Stream};

/// A Utf8Stream over a borrowed string slice.
/// Since the input is already valid UTF-8, this never produces Utf8Result::Invalid.
pub struct StringStream<'a> {
  str: &'a str,
  offset: Cell<usize>,
  buffer: Cell<Option<Utf8Result>>,
}

impl<'a> StringStream<'a> {
  pub fn new(str: &'a str) -> Self {
    Self {
      str,
      offset: Cell::new(0),
      buffer: Cell::new(None),
    }
  }
}

impl<'a> Utf8Stream for StringStream<'a> {
  fn peek(&self) -> Utf8Result {
    if let Some(result) = self.buffer.get() {
      return result;
    }

    let result = match self.str[self.offset.get()..].chars().next() {
      Some(char) => Utf8Result::Char(char),
      None => Utf8Result::Eof,
    };

    self.buffer.set(Some(result));
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
      self.offset.update(|v| v + char.len_utf8());
    }

    result
  }

  fn offset(&self) -> usize {
    self.offset.get()
  }

  fn exhausted(&self) -> bool {
    self.offset.get() >= self.str.len()
  }
}
