use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct Fixture {
  pub path: PathBuf,
  pub contents: String,
}

/// Load all files in a fixture subdirectory as a map of filename to Fixture
pub fn load_fixtures(subdir: &str) -> HashMap<String, Fixture> {
  // TIL: CARGO_MANIFEST_DIR is set to the folder containing the Cargo.toml
  let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("fixtures")
    .join(subdir);

  let mut result = HashMap::new();

  for entry in std::fs::read_dir(&fixtures_dir).unwrap_or_else(|_| {
    panic!(
      "failed to read fixtures directory: {}",
      fixtures_dir.display()
    )
  }) {
    let entry = entry.expect("failed to read directory entry");
    let path = entry.path();
    if path.is_file() {
      let filename = path.file_name().unwrap().to_string_lossy().to_string();
      let contents = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("failed to read fixture: {}", path.display()));
      result.insert(filename, Fixture { path, contents });
    }
  }

  result
}
