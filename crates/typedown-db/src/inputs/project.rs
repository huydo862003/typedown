//! An input struct representing files in a project

use std::{collections::HashMap, path::PathBuf};

use typedown_macros::query_input;

use crate::inputs::FileHandle;

#[query_input]
pub struct Project {
  handles: HashMap<PathBuf, FileHandle>,
}
