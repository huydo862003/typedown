use std::{
  cell::{Cell, RefCell},
  io::{BufReader, Read},
};

use crate::stream::{Utf8Result, Utf8Stream};

/// A Utf8Stream over any `Read` source (file, stdin, etc.) via BufReader.
pub struct FileStream<T: Read> {
  reader: RefCell<BufReader<T>>,
  /// Buffered result for fast repeated peek access
  buffer: Cell<Option<Utf8Result>>,
  /// Current byte offset in the source
  offset: Cell<usize>,
  /// Number of bytes to skip (from invalid UTF-8 recovery)
  skip: Cell<usize>,
}

impl<T: Read> FileStream<T> {
  pub fn new(source: T) -> Self {
    Self {
      reader: RefCell::new(BufReader::new(source)),
      buffer: Cell::new(None),
      offset: Cell::new(0),
      skip: Cell::new(0),
    }
  }
}

impl<T: Read> Utf8Stream for FileStream<T> {
  fn peek(&self) -> Utf8Result {
    if let Some(result) = self.buffer.get() {
      return result;
    }

    let mut bytes = [0u8; 4];
    let mut filled = 0;

    let result = loop {
      match self
        .reader
        .borrow_mut()
        .read(&mut bytes[filled..filled + 1])
      {
        Ok(0) => break Utf8Result::Eof,
        Ok(_) => {
          filled += 1;
          if let Ok(s) = std::str::from_utf8(&bytes[..filled]) {
            let ch = s.chars().next().expect("valid UTF-8 must yield a char");
            break Utf8Result::Char(ch);
          }
          if filled >= 4 {
            // 4 bytes read but still invalid UTF-8
            self.skip.set(filled);
            break Utf8Result::Invalid { len: filled, bytes };
          }
        }
        Err(_) => {
          // I/O error treated as EOF
          break Utf8Result::Eof;
        }
      }
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

    match &result {
      Utf8Result::Char(char) => {
        self.offset.update(|v| v + char.len_utf8());
      }
      Utf8Result::Invalid { .. } => {
        self.offset.update(|v| v + self.skip.get());
        self.skip.set(0);
      }
      Utf8Result::Eof => {}
    }

    result
  }

  fn offset(&self) -> usize {
    self.offset.get()
  }

  fn exhausted(&self) -> bool {
    self.peek();
    matches!(self.buffer.get(), Some(Utf8Result::Eof))
  }
}
