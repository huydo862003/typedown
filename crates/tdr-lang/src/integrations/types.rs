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
