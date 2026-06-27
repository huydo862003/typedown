use std::path::PathBuf;
use std::sync::mpsc;

use ropey::Rope;

use notify::{Event, EventKind};

use lsp_server::{Connection, Message, Notification};
use lsp_types::notification::{
  DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification as _,
};
use lsp_types::{
  DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
  InitializeParams, ServerCapabilities, TextDocumentContentChangeEvent, TextDocumentSyncCapability,
  TextDocumentSyncKind, Uri,
};
use typedown_db::{QueryStorage, TypedownDatabase};
use typedown_lsp::analysis_host::AnalysisHost;
use typedown_lsp::notification;
use typedown_lsp::service;

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
  let mut host = AnalysisHost::new(db, project_dir, watcher_tx)?;

  server_loop(&connection, &mut host, watcher_rx)?;

  io_thread.join()?;
  Ok(())
}

/// The main server loop
fn server_loop(
  connection: &Connection,
  host: &mut AnalysisHost,
  watcher_rx: mpsc::Receiver<notify::Result<Event>>,
) -> anyhow::Result<()> {
  for msg in &connection.receiver {
    // Drain pending file watcher events before handling the next LSP message
    for event in watcher_rx.try_iter() {
      if let Ok(event) = event {
        handle_watcher_event(host, event);
      }
    }

    match msg {
      Message::Request(req) => {
        // Check for shutdown before dispatching
        if connection.handle_shutdown(&req)? {
          break;
        }
        let analysis = host.snapshot();
        let resp = service::dispatch(&analysis, req);
        connection.sender.send(Message::Response(resp))?;
      }
      Message::Notification(note) => {
        // File open/change/close: update the host's tracked state
        handle_notification(host, &note);
        // Push diagnostics after each state change
        let analysis = host.snapshot();
        for notif in notification::diagnostics::publish_diagnostics(&analysis) {
          connection.sender.send(Message::Notification(notif))?;
        }
      }
      Message::Response(_) => {}
    }
  }
  Ok(())
}

fn handle_notification(host: &mut AnalysisHost, note: &Notification) {
  match note.method.as_str() {
    // Editor opened a file: take ownership of its content from the editor buffer.
    DidOpenTextDocument::METHOD => {
      let Ok(params) = serde_json::from_value::<DidOpenTextDocumentParams>(note.params.clone())
      else {
        return;
      };
      host.on_editor_open_file(&params.text_document.uri, params.text_document.text);
    }
    // Editor sent incremental diffs: apply each change to the in-memory rope.
    DidChangeTextDocument::METHOD => {
      let Ok(params) = serde_json::from_value::<DidChangeTextDocumentParams>(note.params.clone())
      else {
        return;
      };
      if let Some(path) = uri_to_path(&params.text_document.uri) {
        let mut rope = host.open_file_content(&path).cloned().unwrap_or_default();
        for change in params.content_changes {
          rope = apply_content_change(rope, change);
        }
        host.on_editor_change_file(path, rope);
      }
    }
    // Editor closed the file: fall back to the on-disk version.
    DidCloseTextDocument::METHOD => {
      let Ok(params) = serde_json::from_value::<DidCloseTextDocumentParams>(note.params.clone())
      else {
        return;
      };
      if let Some(path) = uri_to_path(&params.text_document.uri) {
        host.on_close_file(&path);
      }
    }
    _ => {}
  }
}

fn handle_watcher_event(host: &mut AnalysisHost, event: Event) {
  match event.kind {
    EventKind::Create(_) | EventKind::Modify(_) => {
      for path in event.paths {
        host.on_disk_change(path);
      }
    }
    EventKind::Remove(_) => {
      for path in event.paths {
        host.on_disk_delete(path);
      }
    }
    _ => {}
  }
}

/// Apply a single incremental change to a rope.
/// If the change has no range it is a full replacement.
fn apply_content_change(mut rope: Rope, change: TextDocumentContentChangeEvent) -> Rope {
  let Some(range) = change.range else {
    return Rope::from(change.text);
  };

  let start = rope.line_to_char(range.start.line as usize) + range.start.character as usize;
  let end = rope.line_to_char(range.end.line as usize) + range.end.character as usize;
  rope.remove(start..end);
  rope.insert(start, &change.text);
  rope
}

/// Walk up from `start` until a directory containing `typedown.yaml` or `typedown.yml` is found.
/// Returns `None` if no such directory exists up to the filesystem root.
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
