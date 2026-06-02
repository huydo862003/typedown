//! An input struct representing files in a project

use std::{collections::HashMap, path::PathBuf};

use crate::inputs::FileHandle;

pub struct Project {
  pub handles: HashMap<PathBuf, FileHandle>,
}
