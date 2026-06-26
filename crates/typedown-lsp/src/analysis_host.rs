use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Condvar, Mutex};

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use typedown_db::TypedownDatabase;

use crate::analysis::Analysis;

pub struct AnalysisHost {
  db: TypedownDatabase,
  project_dir: PathBuf,
  snapshot_counter: Arc<(Mutex<usize>, Condvar)>,
  open_files: HashMap<PathBuf, String>, // editor-managed content
  project_files: HashSet<PathBuf>,      // all .tdr files known on disk
  _watcher: RecommendedWatcher,
}

impl AnalysisHost {
  pub fn new(
    db: TypedownDatabase,
    project_dir: PathBuf,
    watcher_tx: Sender<notify::Result<Event>>,
  ) -> Self {
    let mut watcher =
      notify::recommended_watcher(watcher_tx).expect("failed to create file watcher");
    watcher
      .watch(&project_dir, RecursiveMode::Recursive)
      .expect("failed to watch project directory");

    Self {
      db,
      project_dir,
      snapshot_counter: Arc::new((Mutex::new(1), Condvar::new())),
      open_files: HashMap::new(),
      project_files: HashSet::new(),
      _watcher: watcher,
    }
  }

  /// Take a read-only snapshot of the current database state.
  pub fn snapshot(&self) -> Analysis {
    *self.snapshot_counter.0.lock().unwrap() += 1;
    Analysis::new(self.db.clone(), Arc::clone(&self.snapshot_counter))
  }

  /// Cancel all in-flight snapshots, wait for them to finish, then apply a write.
  pub fn write(&mut self, f: impl FnOnce(&mut TypedownDatabase)) {
    self.db.storage.cancelled.store(true, Ordering::Relaxed);

    let mut clones = self.snapshot_counter.0.lock().unwrap();
    while *clones != 1 {
      clones = self.snapshot_counter.1.wait(clones).unwrap();
    }
    drop(clones);

    self.db.storage.cancelled.store(false, Ordering::Relaxed);
    f(&mut self.db);
  }

  /// Called on textDocument/didOpen.
  pub fn on_editor_open_file(&mut self, path: PathBuf, content: String) {
    self.open_files.insert(path, content);
  }

  /// Called on textDocument/didChange.
  pub fn on_editor_change_file(&mut self, path: PathBuf, content: String) {
    self.open_files.insert(path, content);
  }

  /// Called on textDocument/didClose.
  pub fn on_close_file(&mut self, path: &PathBuf) {
    self.open_files.remove(path);
  }

  /// Called by the file watcher for disk changes to non-open files.
  pub fn on_disk_change(&mut self, path: PathBuf) {
    if self.open_files.contains_key(&path) {
      return; // editor owns this file, ignore disk change
    }
    if path.extension().is_some_and(|ext| ext == "tdr") {
      self.project_files.insert(path);
    }
  }

  /// Called by the file watcher when a file is deleted.
  pub fn on_disk_delete(&mut self, path: &PathBuf) {
    if self.open_files.contains_key(path) {
      return;
    }
    self.project_files.remove(path);
  }

  pub fn project_dir(&self) -> &PathBuf {
    &self.project_dir
  }
}
