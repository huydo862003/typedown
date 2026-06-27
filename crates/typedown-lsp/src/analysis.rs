use std::sync::{Arc, Condvar, Mutex};

use typedown_db::TypedownDatabase;
use typedown_db::types::Project;

pub struct Analysis {
  pub(crate) db: TypedownDatabase,
  pub(crate) project: Project,
  snapshot_counter: Arc<(Mutex<usize>, Condvar)>,
}

impl Analysis {
  pub(crate) fn new(
    db: TypedownDatabase,
    project: Project,
    snapshot_counter: Arc<(Mutex<usize>, Condvar)>,
  ) -> Self {
    Self {
      db,
      project,
      snapshot_counter,
    }
  }
}

impl Drop for Analysis {
  fn drop(&mut self) {
    *self.snapshot_counter.0.lock().unwrap() -= 1;
    self.snapshot_counter.1.notify_all();
  }
}
