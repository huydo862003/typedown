use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use lsp_server::Connection;
use lsp_types::{
  CompletionOptions, HoverProviderCapability, InitializeParams, OneOf, SemanticTokensFullOptions,
  SemanticTokensLegend, SemanticTokensOptions, SemanticTokensServerCapabilities,
  ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
  Uri,
};
use typedown_incremental::{CacheSession, SerializableQueryDatabase};
use typedown_lang::db::{QueryStorage, TypedownDatabase};
use typedown_lsp::analysis_host::AnalysisHost;
use typedown_lsp::server::Server;
use typedown_lsp::service::semantic_tokens;

// The entrypoint
fn main() -> anyhow::Result<()> {
  let (connection, io_thread) = Connection::stdio();

  // Capabilities of the server
  let capabilities = ServerCapabilities {
    text_document_sync: Some(TextDocumentSyncCapability::Options(
      TextDocumentSyncOptions {
        // Required for clients to send didOpen, which triggers semantic token requests.
        // See: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocumentSyncOptions
        open_close: Some(true),
        change: Some(TextDocumentSyncKind::INCREMENTAL),
        ..Default::default()
      },
    )),
    hover_provider: Some(HoverProviderCapability::Simple(true)),
    completion_provider: Some(CompletionOptions::default()),
    definition_provider: Some(OneOf::Left(true)),
    semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
      SemanticTokensOptions {
        legend: SemanticTokensLegend {
          token_types: semantic_tokens::token_types(),
          token_modifiers: semantic_tokens::token_modifiers(),
        },
        full: Some(SemanticTokensFullOptions::Bool(true)),
        ..Default::default()
      },
    )),
    ..Default::default()
  };

  // Handshake with the capabilities and get back the client
  let init_params: InitializeParams =
    serde_json::from_value(connection.initialize(serde_json::to_value(capabilities)?)?)?;

  // Lookup the project root
  let workspace_dir = init_params
    .workspace_folders
    .and_then(|folders| folders.into_iter().next())
    .and_then(|folder| uri_to_path(&folder.uri))
    .unwrap_or_else(|| PathBuf::from("."));
  let project_dir = find_project_root(&workspace_dir).unwrap_or(workspace_dir);

  // Load incremental cache from previous session
  let cache_dir = project_dir.join(".typedown/cache");
  let (session, serialized) = CacheSession::open(&cache_dir).unwrap_or_else(|_| {
    // If cache dir is inaccessible, proceed without cache
    (CacheSession::empty(), None)
  });

  let storage = match serialized {
    Some(data) => {
      match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let arc = QueryStorage::from_serialized(data);
        Arc::try_unwrap(arc).unwrap_or_else(|arc| (*arc).clone())
      })) {
        Ok(storage) => storage,
        Err(_) => {
          eprintln!("Failed to load incremental cache, starting fresh");
          let _ = std::fs::remove_dir_all(&cache_dir);
          QueryStorage::default()
        }
      }
    }
    None => QueryStorage::default(),
  };
  let db = TypedownDatabase { storage };

  let host = AnalysisHost::new(db, project_dir)?;

  let host = Server::new(connection, host, init_params.capabilities).run()?;

  // Save incremental cache on shutdown
  let db = host.into_db();
  let revision = db.storage.revision.load(Ordering::Acquire) as u64;
  let serialized = db.dump();
  if let Err(err) = session.finalize(&serialized, revision) {
    eprintln!("Failed to save incremental cache: {}", err);
  }

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
