use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::{fs, io};

use typedown_lang::db::types::{AssetKind, FileHandle, FileMetadata};
use typedown_lang::db::utils::is_content_file;

pub fn disk_handle(path: &PathBuf) -> Option<FileHandle> {
  let meta = fs::metadata(path).ok()?;
  let mtime = meta.modified().ok()?;
  let ctime = meta.created().unwrap_or(mtime);
  Some(FileHandle::Path(
    path.clone(),
    FileMetadata { mtime, ctime },
  ))
}

pub fn scan_project_files(root: &PathBuf) -> io::Result<HashSet<PathBuf>> {
  let mut files = HashSet::new();
  scan_dir(root, root, &mut files)?;
  Ok(files)
}

fn scan_dir(root: &PathBuf, dir: &PathBuf, files: &mut HashSet<PathBuf>) -> io::Result<()> {
  for entry in fs::read_dir(dir)? {
    let entry = entry?;
    let path = entry.path();
    if path.is_dir() {
      scan_dir(root, &path, files)?;
    } else if is_content_file(&path)
      || is_asset_file(&path)
      || (dir == root && is_vault_config(&path))
    {
      files.insert(path);
    }
  }
  Ok(())
}

pub fn is_asset_file(path: &Path) -> bool {
  path
    .extension()
    .and_then(|ext| ext.to_str())
    .and_then(AssetKind::from_extension)
    .is_some()
}

pub fn is_vault_config(path: &Path) -> bool {
  matches!(
    path.file_name().and_then(|name| name.to_str()),
    Some("typedown.yaml") | Some("typedown.yml")
  )
}
