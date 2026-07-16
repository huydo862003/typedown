use std::path::{Path, PathBuf};

use lsp_types::Uri;

pub fn path_to_uri(path: &Path, scheme: &str) -> Uri {
  let uri_str = format!("{scheme}://{}", path.display());
  uri_str
    .parse()
    .unwrap_or_else(|_| "file:///".parse().unwrap())
}

pub fn uri_to_path(uri: &Uri) -> Option<PathBuf> {
  let path = uri.path().as_str();
  if path.is_empty() {
    log::warn!("URI has empty path: {}", uri.as_str());
    return None;
  }
  Some(PathBuf::from(path))
}

pub fn uri_scheme(uri: &Uri) -> &str {
  let s = uri.as_str();
  s.find(':').map(|i| &s[..i]).unwrap_or("file")
}
