use lsp_server::Notification;
use lsp_types::PublishDiagnosticsParams;
use lsp_types::notification::{Notification as _, PublishDiagnostics};

use typedown_db::derived::get_vault_config::get_vault_config;

use crate::analysis::Analysis;
use crate::utils::uri::path_to_uri;

use super::to_lsp_diagnostic;

pub fn publish_diagnostics(analysis: &Analysis) -> Vec<Notification> {
  let db = &analysis.db;
  let project = analysis.project;
  let root = project.root_dir(db);

  // Find the vault config file path
  let config_path = [root.join("typedown.yaml"), root.join("typedown.yml")]
    .into_iter()
    .find(|p| project.files(db).contains_key(p));

  let Some(config_path) = config_path else {
    return vec![];
  };

  let rope = match analysis.file_rope(&config_path) {
    Some(rope) => rope,
    None => return vec![],
  };

  let config_result = get_vault_config(db, project);
  let lsp_diags = config_result
    .diagnostics(db)
    .iter()
    .filter_map(|diag| to_lsp_diagnostic(diag, &rope))
    .collect();

  let scheme = analysis
    .scheme_map
    .get(&config_path)
    .map(String::as_str)
    .unwrap_or("file");
  let uri = path_to_uri(&config_path, scheme);

  vec![Notification::new(
    PublishDiagnostics::METHOD.to_string(),
    PublishDiagnosticsParams {
      uri,
      diagnostics: lsp_diags,
      version: None,
    },
  )]
}
