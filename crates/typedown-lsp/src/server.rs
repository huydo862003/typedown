use std::sync::mpsc;

use ropey::Rope;

use lsp_server::{Connection, Message, Notification};
use lsp_types::notification::{
  DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Notification as _,
};
use lsp_types::{
  DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
  TextDocumentContentChangeEvent,
};
use notify::{Event, EventKind};

use crate::analysis_host::AnalysisHost;
use crate::notification;
use crate::service;
use crate::utils::uri::uri_to_path;

pub struct Server {
  connection: Connection,
  host: AnalysisHost,
  watcher_rx: mpsc::Receiver<notify::Result<Event>>,
}

impl Server {
  pub fn new(
    connection: Connection,
    host: AnalysisHost,
    watcher_rx: mpsc::Receiver<notify::Result<Event>>,
  ) -> Self {
    Self {
      connection,
      host,
      watcher_rx,
    }
  }

  /// Run the server event loop until the client sends a shutdown request.
  pub fn run(mut self) -> anyhow::Result<()> {
    for msg in &self.connection.receiver {
      // Drain pending file-watcher events before handling the next LSP message.
      for event in self.watcher_rx.try_iter() {
        if let Ok(event) = event {
          handle_watcher_event(&mut self.host, event);
        }
      }

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
