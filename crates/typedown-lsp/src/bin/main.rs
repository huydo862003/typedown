use std::path::PathBuf;
use std::sync::mpsc;

use lsp_server::Connection;
use lsp_types::{
  InitializeParams, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
};
use typedown_db::{QueryStorage, TypedownDatabase};
use typedown_lsp::analysis_host::AnalysisHost;
use typedown_lsp::server::Server;

// The entrypoint
fn main() -> anyhow::Result<()> {
  let (connection, io_thread) = Connection::stdio();

  // Capabilities of the server
  // Curently only support syncing documents
  let capabilities = ServerCapabilities {
    text_document_sync: Some(TextDocumentSyncCapability::Kind(
      TextDocumentSyncKind::INCREMENTAL,
    )),
    ..Default::default()
  };

  // Handshake with the capabilities and get back the client
  let init_params: InitializeParams = serde_json::from_value(
    connection.initialize(serde_json::json!({ "capabilities": capabilities }))?,
  )?;

  // Lookup the project root
  let workspace_dir = init_params
    .workspace_folders
    .and_then(|folders| folders.into_iter().next())
    .and_then(|folder| uri_to_path(&folder.uri))
    .unwrap_or_else(|| PathBuf::from("."));
  let project_dir = find_project_root(&workspace_dir).unwrap_or(workspace_dir);

  let db = TypedownDatabase {
    storage: QueryStorage::default(),
  };

  let (watcher_tx, watcher_rx) = mpsc::channel();
  let host = AnalysisHost::new(db, project_dir, watcher_tx)?;

  Server::new(connection, host, watcher_rx).run()?;

  io_thread.join()?;
  Ok(())
}

/// Walk up from `start` until a directory containing `typedown.yaml` or `typedown.yml` is found.
fn find_project_root(start: &PathBuf) -> Option<PathBuf> {
  let mut current = start.as_path();
  loop {
    if current.join("typedown.yaml").exists() || current.join("typedown.yml").exists() {
      return Some(current.to_path_buf());
    }
    current = current.parent()?;
  }
}

fn uri_to_path(uri: &Uri) -> Option<PathBuf> {
  let path = uri.path().as_str();
  if path.is_empty() {
    return None;
  }
  Some(PathBuf::from(path))
}
