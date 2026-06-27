use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Condvar, Mutex};
use std::{fs, io};

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use ropey::Rope;
use typedown_db::TypedownDatabase;
use typedown_db::inputs::{FileHandle, Project};

use crate::analysis::Analysis;

pub struct AnalysisHost {
  db: TypedownDatabase,
  project: Project,
  project_dir: PathBuf,
  snapshot_counter: Arc<(Mutex<usize>, Condvar)>,
  open_files: HashMap<PathBuf, Rope>, // editor-managed content
  project_files: HashSet<PathBuf>,    // all .tdr files known on disk
  _watcher: RecommendedWatcher,
}

impl AnalysisHost {
  pub fn new(
    db: TypedownDatabase,
    project_dir: PathBuf,
    watcher_tx: Sender<notify::Result<Event>>,
  ) -> io::Result<Self> {
    let mut watcher =
      notify::recommended_watcher(watcher_tx).expect("failed to create file watcher");
    watcher
      .watch(&project_dir, RecursiveMode::Recursive)
      .expect("failed to watch project directory");

    // Scan project directory for .tdr files
    let project_files = scan_project_files(&project_dir)?;

    let handles = build_handles(&project_files, &HashMap::new());
    let project = Project::new(&db, project_dir.clone(), handles);

    Ok(Self {
      db,
      project,
      project_dir,
      snapshot_counter: Arc::new((Mutex::new(1), Condvar::new())),
      open_files: HashMap::new(),
      project_files,
      _watcher: watcher,
    })
  }

  /// Take a read-only snapshot of the current database state.
  pub fn snapshot(&self) -> Analysis {
    *self.snapshot_counter.0.lock().unwrap() += 1;
    Analysis::new(
      self.db.clone(),
      self.project,
      Arc::clone(&self.snapshot_counter),
    )
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

  fn sync_handles(&mut self) {
    let handles = build_handles(&self.project_files, &self.open_files);
    let project = self.project;
    self.write(move |db| {
      project.set_handles(db, handles);
    });
  }

  /// Called on textDocument/didOpen.
  pub fn on_editor_open_file(&mut self, path: PathBuf, content: String) {
    self.open_files.insert(path, Rope::from(content));
    self.sync_handles();
  }

  /// Called on textDocument/didChange.
  pub fn on_editor_change_file(&mut self, path: PathBuf, rope: Rope) {
    self.open_files.insert(path, rope);
    self.sync_handles();
  }

  /// Called on textDocument/didClose. Falls back to disk version.
  pub fn on_close_file(&mut self, path: &PathBuf) {
    self.open_files.remove(path);
    self.sync_handles();
  }

  /// Called by the file watcher for disk changes to non-open files.
  pub fn on_disk_change(&mut self, path: PathBuf) {
    if self.open_files.contains_key(&path) {
      return; // editor owns this file, ignore disk change
    }
    if is_tdr_file(&path)
      || (path.parent().is_some_and(|p| p == self.project_dir) && is_vault_config(&path))
    {
      self.project_files.insert(path);
      self.sync_handles();
    }
  }

  /// Called by the file watcher when a file is deleted.
  pub fn on_disk_delete(&mut self, path: PathBuf) {
    if self.open_files.contains_key(&path) {
      return;
    }
    if self.project_files.remove(&path) {
      self.sync_handles();
    }
  }

  pub fn open_file_content(&self, path: &PathBuf) -> Option<&Rope> {
    self.open_files.get(path)
  }

  pub fn project_dir(&self) -> &PathBuf {
    &self.project_dir
  }
}

/// From the profile + open file handles to a hash map
fn build_handles(
  project_files: &HashSet<PathBuf>,
  open_files: &HashMap<PathBuf, Rope>,
) -> HashMap<PathBuf, FileHandle> {
  let mut handles: HashMap<PathBuf, FileHandle> = project_files
    .iter()
    .map(|path| (path.clone(), FileHandle::Path(path.clone())))
    .collect();

  // Editor content overrides disk for open files
  for (path, rope) in open_files {
    handles.insert(path.clone(), FileHandle::Content(rope.to_string()));
  }

  handles
}

/// Read all relevant project files (based on is_tracked_file)
fn scan_project_files(root: &PathBuf) -> io::Result<HashSet<PathBuf>> {
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
    } else if is_tdr_file(&path) || (dir == root && is_vault_config(&path)) {
      files.insert(path);
    }
  }
  Ok(())
}

fn is_tdr_file(path: &PathBuf) -> bool {
  path.extension().is_some_and(|ext| ext == "tdr")
}

fn is_vault_config(path: &PathBuf) -> bool {
  matches!(
    path.file_name().and_then(|n| n.to_str()),
    Some("typedown.yaml") | Some("typedown.yml")
  )
}
