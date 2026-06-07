//! Tracked query to get all files in a project

use std::{collections::HashMap, path::PathBuf};

use crate::{
  TypedownDatabase,
  inputs::{File, Project},
};

pub fn get_files(
  db: &TypedownDatabase,
  project: Project,
) -> HashMap<PathBuf, File> {
  project
    .handles(db)
    .into_iter()
    .map(|(path, handle)| (path, File::new(db, handle)))
    .collect()
}
