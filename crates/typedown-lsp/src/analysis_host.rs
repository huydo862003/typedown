use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Condvar, Mutex};
use std::time::SystemTime;
use std::{fs, io};

use lsp_types::Uri;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use ropey::Rope;
use typedown_db::TypedownDatabase;
use typedown_db::inputs::{File, FileHandle, Project};

use crate::analysis::Analysis;
use crate::utils::uri::{uri_scheme, uri_to_path};

pub struct AnalysisHost {
  db: TypedownDatabase,
  project: Project,
  project_dir: PathBuf,
  snapshot_counter: Arc<(Mutex<usize>, Condvar)>,
  open_files: HashMap<PathBuf, Rope>,   // editor-managed content
  scheme_map: HashMap<PathBuf, String>, // URI scheme per path, set when editor opens a file
  project_files: HashSet<PathBuf>,      // all tracked files known on disk
  file_map: HashMap<PathBuf, File>,     // stable File IDs, one per tracked path
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

    // Create stable File IDs and initial project
    let mut file_map = HashMap::new();
    let mut files = HashMap::new();
    for path in &project_files {
      let handle = disk_handle(path);
      let file = File::new(&db, handle);
      file_map.insert(path.clone(), file);
      files.insert(path.clone(), file);
    }
    let project = Project::new(&db, project_dir.clone(), files);

    Ok(Self {
      db,
      project,
      project_dir,
      snapshot_counter: Arc::new((Mutex::new(1), Condvar::new())),
      open_files: HashMap::new(),
      scheme_map: HashMap::new(),
      project_files,
      file_map,
      _watcher: watcher,
    })
  }

  /// Take a read-only snapshot of the current database state.
  pub fn snapshot(&self) -> Analysis {
    *self.snapshot_counter.0.lock().unwrap() += 1;
    Analysis::new(
      self.db.clone(),
      self.project,
      self.scheme_map.clone(),
      self.open_files.clone(),
      Arc::clone(&self.snapshot_counter),
    )
  }

  /// Cancel all in-flight snapshots, wait for them to finish, then apply a write.
  pub fn write<R>(&mut self, f: impl FnOnce(&mut TypedownDatabase) -> R) -> R {
    self.db.storage.cancelled.store(true, Ordering::Relaxed);

    let mut clones = self.snapshot_counter.0.lock().unwrap();
    while *clones != 1 {
      clones = self.snapshot_counter.1.wait(clones).unwrap();
    }
    drop(clones);

    self.db.storage.cancelled.store(false, Ordering::Relaxed);
    f(&mut self.db)
  }

  fn sync_files(&mut self) {
    // Compute desired handles for all tracked paths
    let mut desired: HashMap<PathBuf, FileHandle> = self
      .project_files
      .iter()
      .map(|path| (path.clone(), disk_handle(path)))
      .collect();
    for (path, rope) in &self.open_files {
      desired.insert(path.clone(), FileHandle::Content(rope.to_string()));
    }

    let project = self.project;
    let old_file_map = std::mem::take(&mut self.file_map);

    let new_file_map = self.write(|db| {
      let mut file_map = HashMap::new();
      let mut files = HashMap::new();

      for (path, handle) in desired {
        let file = if let Some(existing) = old_file_map.get(&path) {
          // Reuse stable ID, update handle for invalidation
          existing.set_handle(db, handle);
          *existing
        } else {
          File::new(db, handle)
        };
        files.insert(path.clone(), file);
        file_map.insert(path, file);
      }

      project.set_files(db, files);
      file_map
    });

    self.file_map = new_file_map;
  }

  /// Called on textDocument/didOpen.
  pub fn on_editor_open_file(&mut self, uri: &Uri, content: String) {
    if let Some(path) = uri_to_path(uri) {
      let scheme = uri_scheme(uri).to_string();
      self.scheme_map.insert(path.clone(), scheme);
      self.open_files.insert(path, Rope::from(content));
      self.sync_files();
    }
  }

  /// Called on textDocument/didChange.
  pub fn on_editor_change_file(&mut self, path: PathBuf, rope: Rope) {
    self.open_files.insert(path, rope);
    self.sync_files();
  }

  /// Called on textDocument/didClose. Falls back to disk version.
  pub fn on_close_file(&mut self, path: &PathBuf) {
    self.open_files.remove(path);
    self.sync_files();
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
      self.sync_files();
    }
  }

  /// Called by the file watcher when a file is deleted.
  pub fn on_disk_delete(&mut self, path: PathBuf) {
    if self.open_files.contains_key(&path) {
      return;
    }
    if self.project_files.remove(&path) {
      self.sync_files();
    }
  }

  pub fn open_file_content(&self, path: &PathBuf) -> Option<&Rope> {
    self.open_files.get(path)
  }

  pub fn project_dir(&self) -> &PathBuf {
    &self.project_dir
  }
}

fn disk_handle(path: &PathBuf) -> FileHandle {
  let mtime = fs::metadata(path)
    .and_then(|meta| meta.modified())
    .unwrap_or(SystemTime::UNIX_EPOCH);
  FileHandle::Path(path.clone(), mtime)
}

/// Read all relevant project files
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
    path.file_name().and_then(|name| name.to_str()),
    Some("typedown.yaml") | Some("typedown.yml")
  )
}
