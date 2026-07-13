use std::{
  fmt::Debug,
  path::{Path, PathBuf},
  sync::{Arc, atomic::Ordering},
};

use anyhow::Error;
use dashmap::DashMap;
use typedown_incremental::{CacheSession, QueryStorage, SerializableQueryDatabase};
use typedown_lang::db::TypedownDatabase;

use crate::analysis_host::AnalysisHost;

pub struct ProjectEntry {
  root_dir: PathBuf,
  cache: CacheSession,
  host: AnalysisHost,
}

impl Debug for ProjectEntry {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.root_dir.to_str().unwrap_or("<unknown path>"));
    Ok(())
  }
}

pub struct Multiproject {
  projects: DashMap<PathBuf, Arc<ProjectEntry>>, // map from project root dir to the corresponding cache sessions
}

impl Multiproject {
  fn projects(&self) -> impl Iterator<Item = Arc<ProjectEntry>> {
    self.projects.iter().map(|e| e.value().clone())
  }

  fn load_project(&self, project_dir: &Path) -> anyhow::Result<Arc<ProjectEntry>> {
    let cache_dir = project_dir.join(".typedown/cache");

    let (session, serialized) = CacheSession::open(&cache_dir).unwrap_or_else(|_| {
      // If cache dir is inaccessible, proceed without cache
      (CacheSession::empty(), None)
    });

    let storage = match serialized {
      Some(data) => {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
          let arc = QueryStorage::from_serialized(data);
          Arc::try_unwrap(arc).unwrap_or_else(|arc| (*arc).clone())
        })) {
          Ok(storage) => storage,
          Err(_) => {
            eprintln!("Failed to load incremental cache, starting fresh");
            let _ = std::fs::remove_dir_all(&cache_dir);
            QueryStorage::default()
          }
        }
      }
      None => QueryStorage::default(),
    };

    let db = TypedownDatabase { storage };

    let host = AnalysisHost::new(db, project_dir.into())?;

    let entry = Arc::new(ProjectEntry {
      root_dir: project_dir.into(),
      host,
      cache: session,
    });

    self.projects.insert(project_dir.into(), entry.clone());

    Ok(entry)
  }

  fn load_nearest_project(&self, nested_dir: &Path) -> anyhow::Result<Arc<ProjectEntry>> {
    let project_dir = find_project_root(&nested_dir)?;
    self.load_project(&project_dir)
  }

  /// NOTE: we should only save when the main thread is running
  fn save(mut self) {
    let projects = std::mem::take(&mut self.projects);
    for project in projects.into_iter().map(|e| e.1) {
      let ProjectEntry {
        host,
        cache: session,
        ..
      } = Arc::try_unwrap(project).expect("Only one thread can save");

      let db = host.into_db();

      let revision = db.storage.revision.load(Ordering::Acquire) as u64;
      let serialized = db.dump();
      if let Err(err) = session.finalize(&serialized, revision) {
        eprintln!("Failed to save incremental cache: {}", err);
      }
    }
  }
}

/// Walk up from `start` until a directory containing `typedown.yaml` or `typedown.yml` is found.
fn find_project_root(start: &Path) -> anyhow::Result<PathBuf> {
  let mut current = start;
  loop {
    if current.join("typedown.yaml").exists() || current.join("typedown.yml").exists() {
      return Ok(current.to_path_buf());
    }
    let parent = current.parent();
    if let Some(p) = parent {
      current = p;
    } else {
      return Err(Error::msg(format!(
        "Project root containing {} not found",
        start.to_str().unwrap_or("<unknown path>"),
      )));
    }
  }
}
