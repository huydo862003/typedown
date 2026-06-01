//! An input salsa struct representing files in a project

use std::{collections::HashMap, path::PathBuf};

use crate::inputs::FileHandle;

/// A project input struct
#[salsa::input]
pub struct Project {
  pub handles: HashMap<PathBuf, FileHandle>,
}
