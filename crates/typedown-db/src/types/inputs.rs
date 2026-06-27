//! Input types for the incremental database

use std::{collections::HashMap, fs, io, path::PathBuf, time::SystemTime};

use typedown_macros::query_input;
use typedown_types::{file_stream::FileStream, stream::Utf8Stream};

/// Types of file-handle: path-based (with mtime for invalidation) or editor-managed content.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum FileHandle {
  /// A file on disk. `mtime` is used to detect changes without reading content eagerly.
  Path(PathBuf, SystemTime),
  /// Content provided directly by the editor buffer.
  Content(String),
}

impl FileHandle {
  pub fn open(&self) -> io::Result<Box<dyn Utf8Stream>> {
    match self {
      FileHandle::Path(path, _) => {
        let file = fs::File::open(path)?;
        Ok(Box::new(FileStream::new(file)))
      }
      FileHandle::Content(content) => {
        let cursor = io::Cursor::new(content.as_bytes().to_vec());
        Ok(Box::new(FileStream::new(cursor)))
      }
    }
  }

  /// Return the path for a disk-backed handle, if any.
  pub fn path(&self) -> Option<&PathBuf> {
    match self {
      FileHandle::Path(path, _) => Some(path),
      FileHandle::Content(_) => None,
    }
  }
}

/// A file input struct
#[query_input]
pub struct File {
  handle: FileHandle,
}

/// A project input struct representing files in a project.
/// `files` maps each tracked path to its stable `File` ID.
/// It only changes when files are added or removed, not when their content changes.
#[query_input]
pub struct Project {
  root_dir: PathBuf,
  files: HashMap<PathBuf, File>,
}
