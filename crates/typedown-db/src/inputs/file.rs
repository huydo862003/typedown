//! An input salsa struct representing a file in a project

use std::{fs, io, path::PathBuf};

use typedown_types::{file_stream::FileStream, stream::Utf8Stream};

/// Types of file-handle: Currently, we support path-based and content-based files
#[derive(Clone, PartialEq, Eq)]
pub enum FileHandle {
  Path(PathBuf),
  Content(String),
}

impl FileHandle {
  pub fn open(&self) -> io::Result<Box<dyn Utf8Stream>> {
    match self {
      FileHandle::Path(path) => {
        let file = fs::File::open(path)?;
        Ok(Box::new(FileStream::new(file)))
      }
      FileHandle::Content(content) => {
        let cursor = io::Cursor::new(content.as_bytes().to_vec());
        Ok(Box::new(FileStream::new(cursor)))
      }
    }
  }
}

/// A file input struct
#[salsa::input]
pub struct File {
  handle: FileHandle,
}
