use std::path::PathBuf;

use lsp_server::Connection;
use lsp_types::{
  CompletionOptions, HoverProviderCapability, InitializeParams, OneOf, SemanticTokensFullOptions,
  SemanticTokensLegend, SemanticTokensOptions, SemanticTokensServerCapabilities,
  ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
  Uri,
};
use typedown_lsp::multiproject::Multiproject;
use typedown_lsp::server::Server;
use typedown_lsp::service::semantic_tokens;

// The entrypoint
pub fn main() -> anyhow::Result<()> {
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

  let multiproject = Multiproject::default();

  // Handshake with the capabilities and get back the client
  let init_params: InitializeParams =
    serde_json::from_value(connection.initialize(serde_json::to_value(capabilities)?)?)?;

  // Lookup the project roots
  let initial_dirs = init_params
    .workspace_folders
    .unwrap_or_default()
    .into_iter()
    .map(|folder| {
      uri_to_path(&folder.uri).unwrap_or(std::env::current_dir().unwrap_or(PathBuf::from(".")))
    });

  for dir in initial_dirs {
    multiproject.load_nearest_project(&dir)?;
  }

  let server = Server::new(connection, multiproject, init_params.capabilities);

  server.run()?;

  server.save();

  io_thread.join()?;
  Ok(())
}

fn uri_to_path(uri: &Uri) -> Option<PathBuf> {
  let path = uri.path().as_str();
  if path.is_empty() {
    return None;
  }
  Some(PathBuf::from(path))
}
