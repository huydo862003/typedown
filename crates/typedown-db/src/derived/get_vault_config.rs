//! Tracked query to get the vault configuration from typedown.yaml

use std::path::PathBuf;

use typedown_macros::query_derived;

use crate::{QueryDatabase, TypedownDatabase, inputs::Project};

#[query_derived]
pub struct VaultConfig {
  version: String,
  content_dir: PathBuf,
  schema_dir: PathBuf,
}

#[query_derived]
pub fn get_vault_config(db: &TypedownDatabase, project: Project) -> VaultConfig {
  let root = project.root_dir(db);

  // Prioritize .yaml over .yml
  let yaml_path = root.join("typedown.yaml");
  let yml_path = root.join("typedown.yml");

  let config_path = if yaml_path.exists() {
    yaml_path
  } else if yml_path.exists() {
    yml_path
  } else {
    panic!(
      "no typedown.yaml or typedown.yml found in {}",
      root.display()
    );
  };

  let contents = std::fs::read_to_string(&config_path)
    .unwrap_or_else(|_| panic!("failed to read {}", config_path.display()));
  let mut docs = yaml_rust2::YamlLoader::load_from_str(&contents)
    .unwrap_or_else(|_| panic!("failed to parse {}", config_path.display()));

  let doc = if docs.is_empty() {
    panic!("empty config file: {}", config_path.display());
  } else {
    docs.swap_remove(0)
  };

  let version = doc["version"]
    .as_str()
    .unwrap_or_else(|| panic!("missing 'version' in {}", config_path.display()))
    .to_string();

  let content_dir_str = doc["vault"]["content_dir"]
    .as_str()
    .unwrap_or_else(|| panic!("missing 'vault.content_dir' in {}", config_path.display()));

  let schema_dir_str = doc["vault"]["schema_dir"]
    .as_str()
    .unwrap_or_else(|| panic!("missing 'vault.schema_dir' in {}", config_path.display()));

  let content_dir = root.join(content_dir_str);
  let schema_dir = root.join(schema_dir_str);

  VaultConfig::new(db, version, content_dir, schema_dir)
}
