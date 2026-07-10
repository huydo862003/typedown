use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use typedown_incremental::CacheSession;
use typedown_lang::db::types::{File, FileHandle, Project};
use typedown_lang::db::{QueryStorage, TypedownDatabase};

// Create a fresh database and register the project files
pub fn setup_db_fresh(project_dir: &Path) -> TypedownDatabase {
  let db = TypedownDatabase {
    storage: QueryStorage::default(),
  };
  register_project(&db, project_dir);
  db
}

// Load a database from the serialized cache and register the project files
pub fn setup_db_cached(cache_dir: &Path, project_dir: &Path) -> TypedownDatabase {
  let (_session, data) = CacheSession::open(cache_dir).unwrap();
  let serialized = data.expect("cache should exist");
  let arc = QueryStorage::from_serialized(serialized);
  let db = TypedownDatabase {
    storage: Arc::try_unwrap(arc).unwrap_or_else(|arc| (*arc).clone()),
  };
  register_project(&db, project_dir);
  db
}

// Scan .tdr and config files, create File/Project inputs
fn register_project(db: &TypedownDatabase, project_dir: &Path) {
  let mut files = HashMap::new();
  for path in scan_project_files(project_dir) {
    let handle = FileHandle::Path(
      path.clone(),
      std::fs::metadata(&path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH),
    );
    let file = File::new(db, handle);
    files.insert(path, file);
  }
  Project::new(db, project_dir.to_path_buf(), files);
}

// Collect all .tdr and config files in a project directory
pub fn scan_project_files(dir: &Path) -> Vec<PathBuf> {
  let root = dir;
  let mut result = Vec::new();
  let mut stack = vec![dir.to_path_buf()];
  while let Some(dir) = stack.pop() {
    let Ok(entries) = std::fs::read_dir(&dir) else {
      continue;
    };
    for entry in entries.flatten() {
      let path = entry.path();
      if path.is_dir() {
        stack.push(path);
      } else {
        let ext = path.extension().and_then(|e| e.to_str());
        let name = path.file_name().and_then(|n| n.to_str());
        if ext == Some("tdr")
          || (dir == root && matches!(name, Some("typedown.yaml") | Some("typedown.yml")))
        {
          result.push(path);
        }
      }
    }
  }
  result
}

// Recursively copy a directory tree
pub fn copy_dir_recursive(src: &Path, dst: &Path) {
  std::fs::create_dir_all(dst).unwrap();
  for entry in std::fs::read_dir(src).unwrap().flatten() {
    let s = entry.path();
    let d = dst.join(entry.file_name());
    if s.is_dir() {
      copy_dir_recursive(&s, &d);
    } else {
      std::fs::copy(&s, &d).unwrap();
    }
  }
}
