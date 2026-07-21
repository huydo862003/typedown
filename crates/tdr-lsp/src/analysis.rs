use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};

use ropey::Rope;
use tdr_lang::db::types::Project;
use tdr_lang::db::TypedownDatabase;

pub struct Analysis {
  pub(crate) db: TypedownDatabase,
  pub(crate) project: Project,
  pub(crate) scheme_map: Arc<HashMap<PathBuf, String>>,
  pub(crate) open_files: Arc<HashMap<PathBuf, Rope>>,
  snapshot_counter: Arc<(Mutex<usize>, Condvar)>,
}

impl Analysis {
  pub(crate) fn new(
    db: TypedownDatabase,
    project: Project,
    scheme_map: Arc<HashMap<PathBuf, String>>,
    open_files: Arc<HashMap<PathBuf, Rope>>,
    snapshot_counter: Arc<(Mutex<usize>, Condvar)>,
  ) -> Self {
    Self {
      db,
      project,
      scheme_map,
      open_files,
      snapshot_counter,
    }
  }

  /// Check if a path is inside the schema directory
  pub(crate) fn is_schema_file(&self, path: &Path) -> bool {
    let schema_dir =
      tdr_lang::db::derived::get_vault_config::get_vault_config(&self.db, self.project)
        .schema_dir(&self.db);
    path.starts_with(&schema_dir)
  }

  /// Get the rope for a file: from the editor buffer if open, otherwise read from disk.
  pub(crate) fn file_rope(&self, path: &Path) -> Option<Rope> {
    if let Some(rope) = self.open_files.get(path) {
      return Some(rope.clone());
    }
    let files = self.project.files(&self.db);
    let file = files.get(path)?;
    let mut reader = file.handle(&self.db).open().ok()?;
    let mut content = String::new();
    loop {
      match reader.advance() {
        tdr_types::stream::Utf8Result::Char(ch) => content.push(ch),
        tdr_types::stream::Utf8Result::Invalid { .. } => {}
        tdr_types::stream::Utf8Result::Eof => break,
      }
    }
    Some(Rope::from(content))
  }
}

impl Drop for Analysis {
  fn drop(&mut self) {
    *self.snapshot_counter.0.lock().unwrap() -= 1;
    self.snapshot_counter.1.notify_all();
  }
}
