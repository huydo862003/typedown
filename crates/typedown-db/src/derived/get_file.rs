//! Tracked query to look up a file from a project by path

use std::path::PathBuf;

use crate::{
  TypedownDatabase,
  inputs::{File, Project},
};

pub fn get_file(
  db: &TypedownDatabase,
  project: Project,
  path: PathBuf,
) -> Option<File> {
  let handles = project.handles(db);
  let handle = handles.get(&path)?.clone();
  Some(File::new(db, handle))
}
