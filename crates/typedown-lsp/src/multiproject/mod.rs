use std::{collections::HashMap, path::PathBuf};

use typedown_incremental::CacheSession;

use crate::analysis_host::AnalysisHost;

pub struct ProjectEntry {
  root_dir: PathBuf,
  cache: CacheSession,
  host: AnalysisHost,
}

pub struct Multiproject {
  projects: HashMap<PathBuf, ProjectEntry>, // map from project root dir to the corresponding cache sessions
}

impl Multiproject {
  fn projects(&self) -> impl Iterator<Item = &PathBuf> {
    self.projects.keys()
  }

  fn load_project(&mut self, root_dir: PathBuf) -> ProjectEntry {
    todo!();
  }

  fn load_nearest_project(&mut self, nested_dir: PathBuf) -> ProjectEntry {
    todo!();
  }

  fn close_and_save_project(&mut self, root_dir: PathBuf) {
    todo!();
  }
}
