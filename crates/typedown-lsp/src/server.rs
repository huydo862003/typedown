use ropey::Rope;

use lsp_server::{Connection, Message, Notification, Request, RequestId};
use lsp_types::notification::{
  DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument, DidOpenTextDocument,
  Notification as _,
};
use lsp_types::request::{RegisterCapability, Request as _};
use lsp_types::{
  ClientCapabilities, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
  DidChangeWatchedFilesRegistrationOptions, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
  FileChangeType, FileSystemWatcher, GlobPattern, Registration, RegistrationParams,
  TextDocumentContentChangeEvent, WatchKind,
};

use crate::analysis_host::AnalysisHost;
use crate::notification;
use crate::service;
use crate::utils::uri::uri_to_path;

pub struct Server {
  connection: Connection,
  host: AnalysisHost,
  client_capabilities: ClientCapabilities,
}

impl Server {
  pub fn new(
    connection: Connection,
    host: AnalysisHost,
    client_capabilities: ClientCapabilities,
  ) -> Self {
    Self {
      connection,
      host,
      client_capabilities,
    }
  }

  /// Run the server event loop until the client sends a shutdown request.
  pub fn run(mut self) -> anyhow::Result<AnalysisHost> {
    self.register_file_watcher()?;

    for msg in &self.connection.receiver {
      match msg {
        Message::Request(req) => {
          // Check for shutdown before dispatching
          if self.connection.handle_shutdown(&req)? {
            break;
          }
          let analysis = self.host.snapshot();
          let resp = service::dispatch(&analysis, req);
          self.connection.sender.send(Message::Response(resp))?;
        }
        Message::Notification(note) => {
          // File open/change/close: update the host's tracked state
          handle_notification(&mut self.host, &note);
          // Push diagnostics after each state change
          let analysis = self.host.snapshot();
          for notif in notification::diagnostics::publish_diagnostics(&analysis) {
            self.connection.sender.send(Message::Notification(notif))?;
          }
        }
        Message::Response(_) => {}
      }
    }
    Ok(self.host)
  }

  fn register_file_watcher(&self) -> anyhow::Result<()> {
    let supports_dynamic = self
      .client_capabilities
      .workspace
      .as_ref()
      .and_then(|workspace| workspace.did_change_watched_files.as_ref())
      .and_then(|cap| cap.dynamic_registration)
      .unwrap_or(false);

    if !supports_dynamic {
      return Ok(());
    }

    // Watch all relevant files
    let watchers = vec![
      FileSystemWatcher {
        glob_pattern: GlobPattern::String("**/*.tdr".to_string()),
        kind: Some(WatchKind::all()),
      },
      FileSystemWatcher {
        glob_pattern: GlobPattern::String("**/typedown.yaml".to_string()),
        kind: Some(WatchKind::all()),
      },
      FileSystemWatcher {
        glob_pattern: GlobPattern::String("**/typedown.yml".to_string()),
        kind: Some(WatchKind::all()),
      },
    ];

    let registration = Registration {
      id: "typedown-file-watcher".to_string(),
      method: DidChangeWatchedFiles::METHOD.to_string(),
      register_options: Some(serde_json::to_value(
        DidChangeWatchedFilesRegistrationOptions { watchers },
      )?),
    };

    let req = Request::new(
      RequestId::from("typedown-register-watcher".to_string()),
      RegisterCapability::METHOD.to_string(),
      RegistrationParams {
        registrations: vec![registration],
      },
    );

    self.connection.sender.send(Message::Request(req))?;
    Ok(())
  }
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
    DidChangeWatchedFiles::METHOD => {
      let Ok(params) = serde_json::from_value::<DidChangeWatchedFilesParams>(note.params.clone())
      else {
        return;
      };
      for change in params.changes {
        if let Some(path) = uri_to_path(&change.uri) {
          match change.typ {
            FileChangeType::CREATED | FileChangeType::CHANGED => host.on_disk_change(path),
            FileChangeType::DELETED => host.on_disk_delete(path),
            _ => {}
          }
        }
      }
    }
    _ => {}
  }
}

/// Apply a single incremental change to a rope. If the change has no range it is a full replacement.
pub(crate) fn apply_content_change(mut rope: Rope, change: TextDocumentContentChangeEvent) -> Rope {
  let Some(range) = change.range else {
    return Rope::from(change.text);
  };

  let start = rope.line_to_char(range.start.line as usize) + range.start.character as usize;
  let end = rope.line_to_char(range.end.line as usize) + range.end.character as usize;
  rope.remove(start..end);
  rope.insert(start, &change.text);
  rope
}
