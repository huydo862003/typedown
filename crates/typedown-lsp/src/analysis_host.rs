use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Condvar, Mutex};

use typedown_db::TypedownDatabase;

use crate::analysis::Analysis;

pub struct AnalysisHost {
  db: TypedownDatabase,
  project_dir: PathBuf,
  snapshot_counter: Arc<(Mutex<usize>, Condvar)>,
}

impl AnalysisHost {
  pub fn new(db: TypedownDatabase, project_dir: PathBuf) -> Self {
    Self {
      db,
      project_dir,
      snapshot_counter: Arc::new((Mutex::new(1), Condvar::new())),
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

  pub fn project_dir(&self) -> &PathBuf {
    &self.project_dir
  }
}
