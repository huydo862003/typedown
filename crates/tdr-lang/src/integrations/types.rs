use std::{collections::HashMap, path::PathBuf};

/// Schema identifier: name (file stem) and absolute path to the schema file
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "export", derive(serde::Serialize, serde::Deserialize))]
pub struct SchemaId {
  pub name: String,
  pub path: PathBuf,
}

impl SchemaId {
  pub fn new(name: impl Into<String>, path: PathBuf) -> Self {
    Self {
      name: name.into(),
      path,
    }
  }

  pub fn as_str(&self) -> &str {
    &self.name
  }
}

impl std::fmt::Display for SchemaId {
  fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    fmt.write_str(&self.name)
  }
}

/// Content file identifier: absolute path to a content file
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "export", derive(serde::Serialize, serde::Deserialize))]
pub struct ContentId {
  pub path: PathBuf,
}

impl ContentId {
  pub fn new(path: PathBuf) -> Self {
    Self { path }
  }

  pub fn name(&self) -> &str {
    self
      .path
      .file_stem()
      .and_then(|s| s.to_str())
      .unwrap_or("unknown")
  }
}

/// YAML key identifier (e.g. "name", "age", "type")
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "export", derive(serde::Serialize, serde::Deserialize))]
pub struct YamlKeyId(String);

impl YamlKeyId {
  pub fn new(key: impl Into<String>) -> Self {
    Self(key.into())
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl std::fmt::Display for YamlKeyId {
  fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    fmt.write_str(&self.0)
  }
}

/// YAML value wrapper for typed transport
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "export", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "export", serde(untagged))]
pub enum YamlValue {
  String(String),
  Number(f64),
  Bool(bool),
  List(Vec<YamlValue>),
  Object(HashMap<YamlKeyId, YamlValue>),
  Null,
}
