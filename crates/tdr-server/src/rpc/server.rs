use std::path::{Path, PathBuf};
use std::sync::Arc;

use jsonrpsee::core::{RpcResult, async_trait};
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::types::error::INVALID_PARAMS_CODE;
use jsonrpsee::{PendingSubscriptionSink, SubscriptionMessage, SubscriptionSink};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use tdr_incremental::QueryStorage;
use tdr_lang::db::TypedownDatabase;
use tdr_lang::integrations::export::{ExportedValue, export_resource};
use tokio::sync::broadcast;

use crate::core::analysis_host::AnalysisHost;
use crate::core::utils::fs::{is_tdr_file, is_vault_config};

use super::contract::{
  TdrBuildRpcServer, TdrBuiltResource, TdrFileChangedNotification, TdrFileCreatedNotification,
  TdrFileDeletedNotification, TdrFilePath, TdrFileRenamedNotification,
  TdrRpcSubscriptionCloseResponse,
};

enum FsEvent {
  Created(PathBuf),
  Modified(PathBuf),
  Removed(PathBuf),
}

/// RPC build server that holds a single project and serves build requests
// TIL: Use tokio::sync::RwLock in async contexts, not std::sync::RwLock as std::sync::RwLock blocks the OS thread while waiting, which can deadlock the tokio runtime if the lock is held across an .await point
pub struct RpcServer {
  root_dir: PathBuf,
  host: tokio::sync::RwLock<AnalysisHost>,
  change_tx: broadcast::Sender<TdrFileChangedNotification>,
  create_tx: broadcast::Sender<TdrFileCreatedNotification>,
  delete_tx: broadcast::Sender<TdrFileDeletedNotification>,
  rename_tx: broadcast::Sender<TdrFileRenamedNotification>,
  // Held to keep the watcher alive for the lifetime of the server
  _watcher: RecommendedWatcher,
}

impl RpcServer {
  /// Create a new RPC server and start watching for file changes
  pub fn new(root_dir: PathBuf) -> anyhow::Result<Arc<Self>> {
    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };
    let host = AnalysisHost::new(db, root_dir.clone())?;

    let (change_tx, _) = broadcast::channel(64);
    let (create_tx, _) = broadcast::channel(64);
    let (delete_tx, _) = broadcast::channel(64);
    let (rename_tx, _) = broadcast::channel(64);

    let (fs_tx, fs_rx) = tokio::sync::mpsc::unbounded_channel();

    let _watcher = Self::setup_watcher(&root_dir, fs_tx)?;

    let server = Arc::new(Self {
      root_dir,
      host: tokio::sync::RwLock::new(host),
      change_tx,
      create_tx,
      delete_tx,
      rename_tx,
      _watcher,
    });

    Self::spawn_fs_watcher_task(Arc::clone(&server), fs_rx);

    Ok(server)
  }

  /// Set up the file watcher
  fn setup_watcher(
    root_dir: &Path,
    fs_tx: tokio::sync::mpsc::UnboundedSender<FsEvent>,
  ) -> anyhow::Result<RecommendedWatcher> {
    let mut watcher = notify::recommended_watcher(move |result: Result<Event, notify::Error>| {
      let Ok(event) = result else { return };
      for path in &event.paths {
        if !is_tdr_file(path) && !is_vault_config(path) {
          continue;
        }
        let fs_event = match event.kind {
          EventKind::Create(_) => FsEvent::Created(path.clone()),
          EventKind::Modify(_) => FsEvent::Modified(path.clone()),
          EventKind::Remove(_) => FsEvent::Removed(path.clone()),
          _ => continue,
        };
        let _ = fs_tx.send(fs_event);
      }
    })?;

    watcher.watch(root_dir, RecursiveMode::Recursive)?;
    Ok(watcher)
  }

  fn spawn_fs_watcher_task(server: Arc<Self>, mut fs_rx: tokio::sync::mpsc::UnboundedReceiver<FsEvent>) {
    tokio::spawn(async move {
      while let Some(event) = fs_rx.recv().await {
        let relative = match &event {
          FsEvent::Created(path) | FsEvent::Modified(path) | FsEvent::Removed(path) => path
            .strip_prefix(&server.root_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string(),
        };

        // Update the incremental database
        let mut host = server.host.write().await;
        match &event {
          FsEvent::Created(path) | FsEvent::Modified(path) => {
            host.on_disk_change(path.clone());
          }
          FsEvent::Removed(path) => {
            host.on_disk_delete(path.clone());
          }
        }
        drop(host);

        // Notify subscribers
        match event {
          FsEvent::Created(_) => {
            let _ = server
              .create_tx
              .send(TdrFileCreatedNotification { path: relative });
          }
          FsEvent::Modified(_) => {
            let _ = server
              .change_tx
              .send(TdrFileChangedNotification { path: relative });
          }
          FsEvent::Removed(_) => {
            let _ = server
              .delete_tx
              .send(TdrFileDeletedNotification { path: relative });
          }
        }
      }
    });
  }

  async fn build_file_impl(&self, file_path: &TdrFilePath) -> RpcResult<TdrBuiltResource> {
    // Take snapshot and release the lock before doing export work
    let analysis = self.host.read().await.snapshot();
    let db = &analysis.db;
    let project = analysis.project;

    let path = self.root_dir.join(&file_path.0);
    let files = project.files(db);
    let file = files.get(&path).ok_or_else(|| {
      ErrorObjectOwned::owned(INVALID_PARAMS_CODE, "File not found in project", None::<()>)
    })?;

    let exported = export_resource(db, project, *file).ok_or_else(|| {
      ErrorObjectOwned::owned(INVALID_PARAMS_CODE, "File is not a resource", None::<()>)
    })?;

    // Convert header to JSON
    let header = exported
      .header
      .into_iter()
      .map(|(key, value)| (key, exported_value_to_json(value)))
      .collect::<serde_json::Map<String, serde_json::Value>>();

    Ok(TdrBuiltResource {
      header: serde_json::Value::Object(header),
      content: exported.content,
    })
  }
}

fn exported_value_to_json(value: ExportedValue) -> serde_json::Value {
  match value {
    ExportedValue::String(string) => serde_json::Value::String(string),
    ExportedValue::Number(num) => serde_json::json!(num),
    ExportedValue::Bool(boolean) => serde_json::Value::Bool(boolean),
    ExportedValue::List(items) => {
      serde_json::Value::Array(items.into_iter().map(exported_value_to_json).collect())
    }
    ExportedValue::Object(map) => {
      let obj = map
        .into_iter()
        .map(|(key, val)| (key, exported_value_to_json(val)))
        .collect();
      serde_json::Value::Object(obj)
    }
    ExportedValue::Null => serde_json::Value::Null,
  }
}

/// Accept a subscription and forward broadcast messages to the client
async fn run_subscription<T: Serialize + Clone>(
  pending: PendingSubscriptionSink,
  mut rx: broadcast::Receiver<T>,
) -> TdrRpcSubscriptionCloseResponse {
  let Ok(sink) = pending.accept().await else {
    return TdrRpcSubscriptionCloseResponse::Err("Failed to accept subscription".into());
  };
  while let Ok(notification) = rx.recv().await {
    if !forward(&sink, &notification).await {
      break;
    }
  }
  TdrRpcSubscriptionCloseResponse::Ok
}

async fn forward<T: Serialize>(sink: &SubscriptionSink, value: &T) -> bool {
  let Ok(msg) = SubscriptionMessage::new(sink.method_name(), sink.subscription_id(), value) else {
    return true; // Skip unserializable notification, keep subscription alive
  };
  sink.send(msg).await.is_ok()
}

#[async_trait]
impl TdrBuildRpcServer<(), ()> for RpcServer {
  async fn request_file(&self, file_path: TdrFilePath) -> RpcResult<TdrBuiltResource> {
    self.build_file_impl(&file_path).await
  }

  async fn on_file_changed(
    &self,
    pending: PendingSubscriptionSink,
  ) -> TdrRpcSubscriptionCloseResponse {
    run_subscription(pending, self.change_tx.subscribe()).await
  }

  async fn on_file_created(
    &self,
    pending: PendingSubscriptionSink,
  ) -> TdrRpcSubscriptionCloseResponse {
    run_subscription(pending, self.create_tx.subscribe()).await
  }

  async fn on_file_deleted(
    &self,
    pending: PendingSubscriptionSink,
  ) -> TdrRpcSubscriptionCloseResponse {
    run_subscription(pending, self.delete_tx.subscribe()).await
  }

  async fn on_file_renamed(
    &self,
    pending: PendingSubscriptionSink,
  ) -> TdrRpcSubscriptionCloseResponse {
    run_subscription(pending, self.rename_tx.subscribe()).await
  }
}
