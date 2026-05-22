use std::io::{BufReader, Read};

use crate::stream::{Utf8Result, Utf8Stream};

/// A Utf8Stream over any `Read` source (file, stdin, etc.) via BufReader.
pub struct FileStream<T: Read> {
  reader: BufReader<T>,
  /// Buffered result for fast repeated peek access
  buffer: Option<Utf8Result>,
  /// Current byte offset in the source
  offset: usize,
  /// Number of bytes to skip (from invalid UTF-8 recovery)
  skip: usize,
}

impl<T: Read> FileStream<T> {
  pub fn new(source: T) -> Self {
    Self {
      reader: BufReader::new(source),
      buffer: None,
      offset: 0,
      skip: 0,
    }
  }
}

impl<T: Read> Utf8Stream for FileStream<T> {
  fn peek(&mut self) -> Utf8Result {
    if let Some(result) = self.buffer {
      return result;
    }

    let mut bytes = [0u8; 4];
    let mut filled = 0;

    let result = loop {
      match self.reader.read(&mut bytes[filled..filled + 1]) {
        Ok(0) => break Utf8Result::Eof,
        Ok(_) => {
          filled += 1;
          if let Ok(s) = std::str::from_utf8(&bytes[..filled]) {
            let ch = s.chars().next().expect("valid UTF-8 must yield a char");
            break Utf8Result::Char(ch);
          }
          if filled >= 4 {
            // 4 bytes read but still invalid UTF-8
            let start = self.offset;
            self.skip = filled;
            break Utf8Result::Invalid {
              start_offset: start,
              end_offset: start + filled,
            };
          }
        }
        Err(_) => {
          // I/O error treated as EOF
          break Utf8Result::Eof;
        }
      }
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

    match &result {
      Utf8Result::Char(char) => {
        self.offset += char.len_utf8();
      }
      Utf8Result::Invalid { .. } => {
        self.offset += self.skip;
        self.skip = 0;
      }
      Utf8Result::Eof => {}
    }

    result
  }

  fn offset(&self) -> usize {
    self.offset
  }

  fn exhausted(&self) -> bool {
    matches!(self.buffer, Some(Utf8Result::Eof))
  }
}
