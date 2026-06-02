//! Tracked query to look up a file from a project by path

use std::path::PathBuf;

use crate::{
  Database,
  inputs::{File, Project},
};

pub fn get_file(db: &Database, project: Project, path: PathBuf) -> Option<File> {
  let handles = project.handles(db);
  let handle = handles.get(&path)?.clone();
  Some(File::new(db, handle))
}
