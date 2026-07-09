//! Input types for the incremental database

use std::{collections::HashMap, fs, io, path::PathBuf, time::SystemTime};

use typedown_macros::query_input;

use typedown_types::{file_stream::FileStream, stream::Utf8Stream};

use strum::FromRepr;

use typedown_incremental::{Decodable, Decoder, Encodable, Encoder};

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

#[derive(FromRepr)]
#[repr(u8)]
enum FileHandleTag {
  Path = 0,
  Content = 1,
}

impl Encodable for FileHandle {
  fn encode(&self, buf: &mut Vec<u8>, encoder: &mut Encoder) {
    match self {
      FileHandle::Path(path, mtime) => {
        encoder.emit_u8(buf, FileHandleTag::Path as u8);
        path.encode(buf, encoder);
        let duration = mtime
          .duration_since(SystemTime::UNIX_EPOCH)
          .unwrap_or_default();
        duration.as_secs().encode(buf, encoder);
        (duration.subsec_nanos() as u32).encode(buf, encoder);
      }
      FileHandle::Content(content) => {
        encoder.emit_u8(buf, FileHandleTag::Content as u8);
        content.encode(buf, encoder);
      }
    }
  }
}

impl Decodable for FileHandle {
  fn decode(data: &mut &[u8], decoder: &Decoder) -> Self {
    let tag = decoder.read_u8(data);
    match FileHandleTag::from_repr(tag).expect("unknown FileHandle tag") {
      FileHandleTag::Path => {
        let path = PathBuf::decode(data, decoder);
        let secs = u64::decode(data, decoder);
        let nanos = u32::decode(data, decoder);
        let mtime = SystemTime::UNIX_EPOCH + std::time::Duration::new(secs, nanos);
        FileHandle::Path(path, mtime)
      }
      FileHandleTag::Content => FileHandle::Content(String::decode(data, decoder)),
    }
  }
}

/// A project input struct representing files in a project.
/// `files` maps each tracked path to its stable `File` ID.
/// It only changes when files are added or removed, not when their content changes.
#[query_input]
pub struct Project {
  root_dir: PathBuf,
  files: HashMap<PathBuf, File>,
}
