use lsp_server::Connection;
use lsp_types::{
  CompletionOptions, HoverProviderCapability, InitializeParams, InitializeResult, OneOf,
  SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
  SemanticTokensServerCapabilities, ServerCapabilities, ServerInfo, TextDocumentSyncCapability,
  TextDocumentSyncKind,
};
use typedown_lsp::logger;
use typedown_lsp::multiproject::Multiproject;
use typedown_lsp::server::Server;
use typedown_lsp::service::semantic_tokens;

// The entrypoint
pub fn main() -> anyhow::Result<()> {
  let (connection, io_thread) = Connection::stdio();

  // File logger available immediately, before handshake
  logger::init_file();

  let capabilities = ServerCapabilities {
    text_document_sync: Some(TextDocumentSyncCapability::Kind(
      TextDocumentSyncKind::INCREMENTAL,
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

  // connection.initialize wraps its arg in { "capabilities": ... },
  // so we use initialize_start/initialize_finish to also include serverInfo
  let (init_id, init_params) = connection.initialize_start()?;
  let init_data = serde_json::to_value(InitializeResult {
    capabilities,
    server_info: Some(ServerInfo {
      name: "typedown-lsp".to_string(),
      version: Some(env!("CARGO_PKG_VERSION").to_string()),
    }),
  })?;
  connection.initialize_finish(init_id, init_data)?;
  let init_params: InitializeParams = serde_json::from_value(init_params)?;

  // Upgrade logger to also send window/logMessage after handshake
  logger::set_lsp_sender(connection.sender.clone());

  // Projects are loaded lazily on first didOpen/request via load_nearest_project
  log::info!("Typedown LSP server started");

  let server = Server::new(connection, multiproject, init_params.capabilities);

  server.run()?;

  log::info!("Shutting down, saving cache");
  server.save();

  io_thread.join()?;
  Ok(())
}
