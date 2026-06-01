//! Tracked query to get all files in a project

use std::{collections::HashMap, path::PathBuf};

use crate::inputs::{File, Project};

#[salsa::tracked]
pub fn get_files(db: &dyn salsa::Database, project: Project) -> HashMap<PathBuf, File> {
  project
    .handles(db)
    .into_iter()
    .map(|(path, handle)| (path, File::new(db, handle)))
    .collect()
}
