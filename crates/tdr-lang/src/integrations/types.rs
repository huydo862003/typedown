use std::collections::HashMap;

/// Schema identifier, derived from the schema file stem (e.g. "Person")
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "export", derive(serde::Serialize, serde::Deserialize))]
pub struct SchemaId(String);

impl SchemaId {
  pub fn new(name: impl Into<String>) -> Self {
    Self(name.into())
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl std::fmt::Display for SchemaId {
  fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    fmt.write_str(&self.0)
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
