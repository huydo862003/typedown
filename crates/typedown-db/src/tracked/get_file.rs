//! Tracked query to look up a file from a project by path

use std::path::PathBuf;

use crate::inputs::{File, Project};

#[salsa::tracked]
pub fn get_file(db: &dyn salsa::Database, project: Project, path: PathBuf) -> Option<File> {
  let handles = project.handles(db);
  let handle = handles.get(&path)?.clone();
  Some(File::new(db, handle))
}
