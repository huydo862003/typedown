use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Error;
use ropey::Rope;
use threadpool::ThreadPool;
use tdr_incremental::Cancelled;

use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::{
  DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument, DidOpenTextDocument,
  Notification as NotificationTrait,
};
use lsp_types::request::{RegisterCapability, Request as RequestTrait};
use lsp_types::{
  ClientCapabilities, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
  DidChangeWatchedFilesRegistrationOptions, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
  FileChangeType, FileSystemWatcher, GlobPattern, Registration, RegistrationParams,
  TextDocumentContentChangeEvent, WatchKind,
};

use crate::analysis::Analysis;
use crate::analysis_host::AnalysisHost;
use crate::multiproject::{Multiproject, ProjectEntry};
use crate::notification::diagnostics::{publish_diagnostics, publish_diagnostics_for_file};
use crate::service;
use crate::utils::lsp::{try_extract_path_from_notification, try_extract_path_from_request};
use crate::utils::uri::uri_to_path;

pub struct Server {
  connection: Connection,
  multiproject: Multiproject,
  client_capabilities: ClientCapabilities,
  thread_pool: ThreadPool,
}

impl Server {
  pub fn new(
    connection: Connection,
    multiproject: Multiproject,
    client_capabilities: ClientCapabilities,
  ) -> Self {
    let num_threads = std::thread::available_parallelism()
      .map(|count| count.get().min(4))
      .unwrap_or(2);
    Self {
      connection,
      multiproject,
      client_capabilities,
      thread_pool: ThreadPool::new(num_threads),
    }
  }

  pub fn save(self) {
    // Wait for in-flight requests and diagnostics to finish
    self.thread_pool.join();
    self.multiproject.save();
  }

  /// Run the server event loop until the client sends a shutdown request.
  pub fn run(&self) -> anyhow::Result<()> {
    self.register_file_watcher()?;

    for msg in &self.connection.receiver {
      match msg {
        Message::Request(req) => {
          if let Err(err) = self.handle_request(req) {
            log::error!("Failed to handle request: {err}");
          }
        }
        Message::Notification(note) => {
          if let Err(err) = self.handle_notification(note) {
            log::error!("Failed to handle notification: {err}");
          }
        }
        Message::Response(_) => {}
      }
    }
    Ok(())
  }

  fn handle_request(&self, req: Request) -> anyhow::Result<()> {
    if self.connection.handle_shutdown(&req)? {
      return Ok(());
    }

    // Resolve to the owning project
    let uri = try_extract_path_from_request(&req)?;
    let path = uri_to_path(&uri).ok_or_else(|| Error::msg("Failed to convert URI to path"))?;
    let project_entry = self.multiproject.load_nearest_project(&path)?;

    let analysis = project_entry
      .host
      .read()
      .map_err(|_| Error::msg("project_entry.host RwLock is poisoned"))?
      .snapshot();

    // Dispatch to thread pool so the main loop stays responsive.
    // Cancelled::catch handles the case where a didChange cancels in-flight queries.
    let sender = self.connection.sender.clone();
    let request_id = req.id.clone();
    self.thread_pool.execute(move || {
      let resp = match Cancelled::catch(|| service::dispatch(&analysis, req)) {
        Ok(resp) => resp,
        // A didChange arrived and cancelled this query via the DB's cancelled flag
        Err(_) => Response::new_err(
          request_id,
          lsp_server::ErrorCode::ContentModified as i32,
          "request cancelled: content modified".to_string(),
        ),
      };
      if let Err(err) = sender.send(Message::Response(resp)) {
        log::error!("Failed to send response: {err}");
      }
    });

    Ok(())
  }

  fn handle_notification(&self, note: Notification) -> anyhow::Result<()> {
    // DidChangeWatchedFiles can contain changes spanning multiple projects.
    // Route each change to its own project independently.
    if note.method == DidChangeWatchedFiles::METHOD {
      let params = serde_json::from_value::<DidChangeWatchedFilesParams>(note.params.clone())?;
      // Collect affected projects so we push diagnostics once per project, not per file
      let mut affected_projects = Vec::new();
      for change in params.changes {
        let Some(path) = uri_to_path(&change.uri) else {
          log::warn!(
            "Could not convert watched file URI to path: {}",
            change.uri.as_str()
          );
          continue;
        };
        let project_entry = match self.multiproject.load_nearest_project(&path) {
          Ok(entry) => entry,
          Err(err) => {
            log::warn!(
              "No project found for watched file {}: {err}",
              path.display()
            );
            continue;
          }
        };
        {
          let mut host = project_entry
            .host
            .write()
            .expect("RwLock should not be poisoned");
          match change.typ {
            FileChangeType::CREATED | FileChangeType::CHANGED => host.on_disk_change(path),
            FileChangeType::DELETED => host.on_disk_delete(path),
            _ => {}
          }
        }
        if !affected_projects
          .iter()
          .any(|p: &Arc<ProjectEntry>| p.root_dir == project_entry.root_dir)
        {
          affected_projects.push(project_entry);
        }
      }
      for project_entry in &affected_projects {
        self.send_diagnostics_async(project_entry, None);
      }
      return Ok(());
    }

    // For other notifications, extract the document URI and route to a single project
    let uri = try_extract_path_from_notification(&note)?;
    let path = uri_to_path(&uri).ok_or_else(|| Error::msg("Failed to convert URI to path"))?;
    let project_entry = self.multiproject.load_nearest_project(&path)?;

    let method = note.method.clone();

    let analysis = {
      let mut host = project_entry
        .host
        .write()
        .expect("RwLock should not be poisoned");
      handle_text_notification(&mut host, &note)?;
      host.snapshot()
    };

    // didOpen: All files, so cross-file errors show immediately
    // didChange: Only the changed file, for responsiveness
    // didClose: No diagnostics needed
    if method == DidOpenTextDocument::METHOD {
      self.send_diagnostics_with_snapshot(analysis, None);
    } else if method == DidChangeTextDocument::METHOD {
      self.send_diagnostics_with_snapshot(analysis, Some(path));
    }

    Ok(())
  }

  // Compute and send diagnostics on a worker thread using an existing snapshot
  fn send_diagnostics_with_snapshot(&self, analysis: Analysis, path: Option<PathBuf>) {
    let sender = self.connection.sender.clone();
    self.thread_pool.execute(move || {
      // Silently drop if cancelled by a newer didChange
      let Ok(notifications) = Cancelled::catch(|| match path.as_deref() {
        Some(path) => publish_diagnostics_for_file(&analysis, path),
        None => publish_diagnostics(&analysis),
      }) else {
        return;
      };
      for notif in notifications {
        if let Err(err) = sender.send(Message::Notification(notif)) {
          log::error!("Failed to send diagnostics: {err}");
          break;
        }
      }
    });
  }

  // Take a fresh snapshot and send diagnostics on a worker thread
  fn send_diagnostics_async(&self, project_entry: &ProjectEntry, path: Option<&Path>) {
    let analysis = project_entry
      .host
      .read()
      .expect("RwLock should not be poisoned")
      .snapshot();
    let path = path.map(Path::to_path_buf);
    self.send_diagnostics_with_snapshot(analysis, path);
  }

  /* File watcher */
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

/// Handle text document notifications (open, change, close).
fn handle_text_notification(host: &mut AnalysisHost, note: &Notification) -> anyhow::Result<()> {
  match note.method.as_str() {
    // Editor opened a file: take ownership of its content from the editor buffer.
    DidOpenTextDocument::METHOD => {
      let params = serde_json::from_value::<DidOpenTextDocumentParams>(note.params.clone())?;
      host.on_editor_open_file(&params.text_document.uri, params.text_document.text);
    }
    // Editor sent incremental diffs: apply each change to the in-memory rope.
    DidChangeTextDocument::METHOD => {
      let params = serde_json::from_value::<DidChangeTextDocumentParams>(note.params.clone())?;
      let path = uri_to_path(&params.text_document.uri)
        .ok_or_else(|| Error::msg("Failed to convert URI to path"))?;

      let mut rope = host.open_file_content(&path).cloned().unwrap_or_default();
      for change in params.content_changes {
        rope = apply_content_change(rope, change);
      }
      host.on_editor_change_file(path, rope);
    }
    // Editor closed the file: fall back to the on-disk version.
    DidCloseTextDocument::METHOD => {
      let params = serde_json::from_value::<DidCloseTextDocumentParams>(note.params.clone())?;
      let path = uri_to_path(&params.text_document.uri)
        .ok_or_else(|| Error::msg("Failed to convert URI to path"))?;
      host.on_close_file(&path);
    }
    _ => {}
  };
  Ok(())
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
