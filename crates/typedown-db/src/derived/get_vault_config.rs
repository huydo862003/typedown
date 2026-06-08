//! Tracked query to get the vault configuration from typedown.yaml

use std::path::PathBuf;

use typedown_macros::query_derived;
use typedown_types::diagnostic::Diagnostic;

use crate::{
  QueryDatabase, TypedownDatabase,
  types::{Project, VaultConfigResult},
};

#[query_derived]
pub fn get_vault_config(db: &TypedownDatabase, project: Project) -> VaultConfigResult {
  let root = project.root_dir(db);
  let mut diagnostics = Vec::new();

  // Prioritize .yaml over .yml
  let yaml_path = root.join("typedown.yaml");
  let yml_path = root.join("typedown.yml");

  let config_path = if yaml_path.exists() {
    yaml_path
  } else if yml_path.exists() {
    yml_path
  } else {
    diagnostics.push(Diagnostic::MissingVaultConfig {
      root_dir: root.display().to_string(),
    });
    return VaultConfigResult::new(
      db,
      String::new(),
      PathBuf::new(),
      PathBuf::new(),
      diagnostics,
    );
  };

  let contents = match std::fs::read_to_string(&config_path) {
    Ok(contents) => contents,
    Err(err) => {
      diagnostics.push(Diagnostic::VaultConfigReadError {
        path: config_path.display().to_string(),
        message: err.to_string(),
      });
      return VaultConfigResult::new(
        db,
        String::new(),
        PathBuf::new(),
        PathBuf::new(),
        diagnostics,
      );
    }
  };

  let mut docs = match yaml_rust2::YamlLoader::load_from_str(&contents) {
    Ok(docs) => docs,
    Err(err) => {
      diagnostics.push(Diagnostic::VaultConfigParseError {
        path: config_path.display().to_string(),
        message: err.to_string(),
      });
      return VaultConfigResult::new(
        db,
        String::new(),
        PathBuf::new(),
        PathBuf::new(),
        diagnostics,
      );
    }
  };

  if docs.is_empty() {
    diagnostics.push(Diagnostic::VaultConfigEmpty {
      path: config_path.display().to_string(),
    });
    return VaultConfigResult::new(
      db,
      String::new(),
      PathBuf::new(),
      PathBuf::new(),
      diagnostics,
    );
  }

  let doc = docs.swap_remove(0);
  let path_str = config_path.display().to_string();

  let version = doc["version"]
    .as_str()
    .map(|s| s.to_string())
    .unwrap_or_else(|| {
      diagnostics.push(Diagnostic::VaultConfigMissingField {
        path: path_str.clone(),
        field: "version".to_string(),
      });
      String::new()
    });

  let content_dir = doc["vault"]["content_dir"]
    .as_str()
    .map(|s| root.join(s))
    .unwrap_or_else(|| {
      diagnostics.push(Diagnostic::VaultConfigMissingField {
        path: path_str.clone(),
        field: "vault.content_dir".to_string(),
      });
      PathBuf::new()
    });

  let schema_dir = doc["vault"]["schema_dir"]
    .as_str()
    .map(|s| root.join(s))
    .unwrap_or_else(|| {
      diagnostics.push(Diagnostic::VaultConfigMissingField {
        path: path_str.clone(),
        field: "vault.schema_dir".to_string(),
      });
      PathBuf::new()
    });

  VaultConfigResult::new(db, version, content_dir, schema_dir, diagnostics)
}
