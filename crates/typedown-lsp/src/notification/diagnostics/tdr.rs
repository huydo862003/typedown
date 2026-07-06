use lsp_server::Notification;
use lsp_types::notification::{Notification as _, PublishDiagnostics};
use lsp_types::{Diagnostic, PublishDiagnosticsParams};
use typedown_lang::db::derived::evaluate::evaluate_resource::evaluate_resource;
use typedown_lang::db::derived::name_resolver::file_symbol::file_symbol;
use typedown_lang::db::derived::parse_file::parse_file;
use typedown_lang::syntax::diagnostic::Diagnostic as TdrDiagnostic;

use crate::analysis::Analysis;
use crate::utils::uri::path_to_uri;

use super::to_lsp_diagnostic;

pub fn publish_diagnostics(analysis: &Analysis) -> Vec<Notification> {
  let db = &analysis.db;
  let project = analysis.project;
  let files = project.files(db);

  let mut notifications = Vec::new();

  for (path, file) in &files {
    if path.extension().and_then(|e| e.to_str()) != Some("tdr") {
      continue;
    }

    let rope = match analysis.file_rope(path) {
      Some(rope) => rope,
      None => continue,
    };

    let parse_result = parse_file(db, project, *file);
    let mut tdr_diags: Vec<TdrDiagnostic> = parse_result.diagnostics(db).to_vec();

    if let Some(sym) = file_symbol(db, project, *file).value(db) {
      let eval_result = evaluate_resource(db, sym);
      tdr_diags.extend(eval_result.diagnostics(db).iter().cloned());
    }

    let lsp_diags: Vec<Diagnostic> = tdr_diags
      .iter()
      .filter_map(|diag| to_lsp_diagnostic(diag, &rope))
      .collect();

    let scheme = analysis
      .scheme_map
      .get(path)
      .map(String::as_str)
      .unwrap_or("file");
    let uri = path_to_uri(path, scheme);
    let params = PublishDiagnosticsParams {
      uri,
      diagnostics: lsp_diags,
      version: None,
    };

    notifications.push(Notification::new(
      PublishDiagnostics::METHOD.to_string(),
      params,
    ));
  }

  notifications
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use std::path::PathBuf;
  use std::sync::{Arc, Condvar, Mutex};

  use typedown_lang::db::types::{File, FileHandle, Project};
  use typedown_lang::db::{QueryStorage, TypedownDatabase};

  use crate::analysis::Analysis;

  use super::publish_diagnostics;

  const VAULT_CONFIG: &str = r#"version: "1"
vault:
  content_dir: content
  schema_dir: schemas
"#;
  const SCHEMA_PERSON: &str = r#"---
_type: schema
properties:
  name:
    type: string
  age:
    type: number
---
"#;

  fn setup(content: &str) -> Analysis {
    let root = PathBuf::from("/vault");
    let content_path = root.join("content/file.tdr");

    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };

    let config_file = File::new(&db, FileHandle::Content(VAULT_CONFIG.to_string()));
    let person_file = File::new(&db, FileHandle::Content(SCHEMA_PERSON.to_string()));
    let content_file = File::new(&db, FileHandle::Content(content.to_string()));

    let files = HashMap::from([
      (root.join("typedown.yaml"), config_file),
      (root.join("schemas/Person.tdr"), person_file),
      (content_path, content_file),
    ]);

    let project = Project::new(&db, root, files);
    Analysis::new(
      db,
      project,
      HashMap::new(),
      HashMap::new(),
      Arc::new((Mutex::new(1), Condvar::new())),
    )
  }

  #[test]
  fn no_diagnostics_for_valid_file() {
    let analysis = setup(
      r#"---
_type: Person
name: "Alice"
age: 30
---
"#,
    );
    let notifications = publish_diagnostics(&analysis);
    // Only the content file should have a notification; it must be empty.
    let content_notif = notifications
      .iter()
      .find(|notif| notif.params.to_string().contains("content/file.tdr"));
    if let Some(notif) = content_notif {
      let params: serde_json::Value = serde_json::from_str(&notif.params.to_string()).unwrap();
      let diags = params["diagnostics"].as_array().unwrap();
      assert!(
        diags.is_empty(),
        "valid file should produce no diagnostics, got: {diags:?}"
      );
    }
  }

  #[test]
  fn unresolved_schema_produces_diagnostic() {
    // _type references a schema that does not exist.
    let analysis = setup(
      r#"---
_type: NonExistent
name: "Alice"
---
"#,
    );
    let notifications = publish_diagnostics(&analysis);
    let content_notif = notifications
      .iter()
      .find(|notif| notif.params.to_string().contains("content/file.tdr"));
    let notif = content_notif.expect("expected a notification for the content file");
    let params: serde_json::Value = serde_json::from_str(&notif.params.to_string()).unwrap();
    let diags = params["diagnostics"].as_array().unwrap();
    assert!(
      !diags.is_empty(),
      "unresolved schema should produce at least one diagnostic"
    );
    let codes: Vec<&str> = diags
      .iter()
      .filter_map(|diag| diag["code"].as_str())
      .collect();
    assert!(
      codes.iter().any(|code| *code == "unresolved-schema"),
      "expected an unresolved-schema diagnostic, got codes: {codes:?}"
    );
  }

  #[test]
  fn missing_required_field_produces_diagnostic() {
    // Required field 'age' is absent.
    let analysis = setup(
      r#"---
_type: Person
name: "Alice"
---
"#,
    );
    let notifications = publish_diagnostics(&analysis);
    let content_notif = notifications
      .iter()
      .find(|notif| notif.params.to_string().contains("content/file.tdr"));
    let notif = content_notif.expect("expected a notification for the content file");
    let params: serde_json::Value = serde_json::from_str(&notif.params.to_string()).unwrap();
    let diags = params["diagnostics"].as_array().unwrap();
    assert!(
      !diags.is_empty(),
      "missing required field should produce at least one diagnostic"
    );
    let codes: Vec<&str> = diags
      .iter()
      .filter_map(|diag| diag["code"].as_str())
      .collect();
    assert!(
      codes.iter().any(|code| *code == "missing-required-field"),
      "expected a missing-required-field diagnostic, got codes: {codes:?}"
    );
  }
}
