use std::sync::{Arc, Condvar, Mutex};

use typedown_db::TypedownDatabase;

pub struct Analysis {
  pub(crate) db: TypedownDatabase,
  snapshot_counter: Arc<(Mutex<usize>, Condvar)>,
}

impl Analysis {
  pub(crate) fn new(db: TypedownDatabase, snapshot_counter: Arc<(Mutex<usize>, Condvar)>) -> Self {
    Self {
      db,
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
