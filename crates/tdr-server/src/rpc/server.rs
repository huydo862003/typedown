use std::collections::HashMap;
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
use tdr_lang::db::derived::get_vault_config::get_vault_config;
use tdr_lang::db::derived::name_resolver::file_symbol::file_symbol;
use tdr_lang::db::types::SymbolKind;
use tdr_lang::integrations::export::ExportedValue;
use tdr_lang::integrations::export::export_resource;
use tdr_lang::integrations::types::{SchemaId, YamlKeyId, YamlValue};
use tokio::sync::broadcast;

use crate::core::analysis_host::AnalysisHost;
use crate::core::utils::fs::{is_asset_file, is_tdr_file, is_vault_config};

use super::contract::{
  TdrBuildRpcServer, TdrBuiltResource, TdrContentNotification, TdrFilePath,
  TdrRpcSubscriptionCloseResponse, TdrSchemaInfo, TdrSchemaNotification,
};

enum FsEvent {
  Created(PathBuf),
  Modified(PathBuf),
  Removed(PathBuf),
}

/// RPC build server that holds a single project and serves build requests
// TIL: Use tokio::sync::RwLock in async contexts, not std::sync::RwLock as std::sync::RwLock blocks the OS thread while waiting, which can deadlock the tokio runtime if the lock is held across an .await point
pub struct RpcServer {
  _root_dir: PathBuf,
  host: tokio::sync::RwLock<AnalysisHost>,
  // Content events
  content_changed_tx: broadcast::Sender<TdrContentNotification>,
  content_created_tx: broadcast::Sender<TdrContentNotification>,
  content_deleted_tx: broadcast::Sender<TdrContentNotification>,
  // Schema events
  schema_changed_tx: broadcast::Sender<TdrSchemaNotification>,
  schema_created_tx: broadcast::Sender<TdrSchemaNotification>,
  schema_deleted_tx: broadcast::Sender<TdrSchemaNotification>,
  // Held to keep the watcher alive for the lifetime of the server
  _watcher: RecommendedWatcher,
}

impl RpcServer {
  pub fn new(root_dir: PathBuf) -> anyhow::Result<Arc<Self>> {
    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };
    let host = AnalysisHost::new(db, root_dir.clone())?;

    let (content_changed_tx, _) = broadcast::channel(64);
    let (content_created_tx, _) = broadcast::channel(64);
    let (content_deleted_tx, _) = broadcast::channel(64);
    let (schema_changed_tx, _) = broadcast::channel(64);
    let (schema_created_tx, _) = broadcast::channel(64);
    let (schema_deleted_tx, _) = broadcast::channel(64);

    let (fs_tx, fs_rx) = tokio::sync::mpsc::unbounded_channel();
    let _watcher = Self::setup_watcher(&root_dir, fs_tx)?;

    let server = Arc::new(Self {
      _root_dir: root_dir,
      host: tokio::sync::RwLock::new(host),
      content_changed_tx,
      content_created_tx,
      content_deleted_tx,
      schema_changed_tx,
      schema_created_tx,
      schema_deleted_tx,
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
        if !is_tdr_file(path) && !is_asset_file(path) && !is_vault_config(path) {
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

  fn spawn_fs_watcher_task(
    server: Arc<Self>,
    mut fs_rx: tokio::sync::mpsc::UnboundedReceiver<FsEvent>,
  ) {
    tokio::spawn(async move {
      while let Some(event) = fs_rx.recv().await {
        let path = match &event {
          FsEvent::Created(path) | FsEvent::Modified(path) | FsEvent::Removed(path) => path.clone(),
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
        let analysis = host.snapshot();
        drop(host);

        // Classify as content or schema and notify subscribers
        let db = &analysis.db;
        let project = analysis.project;
        let config = get_vault_config(db, project);
        let content_dir = config.content_dir(db);
        let schema_dir = config.schema_dir(db);

        if path.starts_with(&content_dir) {
          let relative = path
            .strip_prefix(&content_dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
          let notification = TdrContentNotification { path: relative };
          match event {
            FsEvent::Created(_) => {
              let _ = server.content_created_tx.send(notification);
            }
            FsEvent::Modified(_) => {
              let _ = server.content_changed_tx.send(notification);
            }
            FsEvent::Removed(_) => {
              let _ = server.content_deleted_tx.send(notification);
            }
          }
        } else if path.starts_with(&schema_dir) {
          let schema = SchemaId::new(
            path
              .file_stem()
              .and_then(|s| s.to_str())
              .unwrap_or("unknown"),
          );
          let notification = TdrSchemaNotification { schema };
          match event {
            FsEvent::Created(_) => {
              let _ = server.schema_created_tx.send(notification);
            }
            FsEvent::Modified(_) => {
              let _ = server.schema_changed_tx.send(notification);
            }
            FsEvent::Removed(_) => {
              let _ = server.schema_deleted_tx.send(notification);
            }
          }
        }
      }
    });
  }

  async fn build_file_impl(&self, file_path: &TdrFilePath) -> RpcResult<TdrBuiltResource> {
    let analysis = self.host.read().await.snapshot();
    let db = &analysis.db;
    let project = analysis.project;

    let content_dir = get_vault_config(db, project).content_dir(db);
    let path = content_dir.join(&file_path.0);
    let files = project.files(db);
    let file = files.get(&path).ok_or_else(|| {
      ErrorObjectOwned::owned(INVALID_PARAMS_CODE, "File not found in project", None::<()>)
    })?;

    let exported = export_resource(db, project, *file).ok_or_else(|| {
      ErrorObjectOwned::owned(INVALID_PARAMS_CODE, "File is not a resource", None::<()>)
    })?;

    let header = exported
      .header
      .into_iter()
      .map(|(key, value)| (YamlKeyId::new(key), exported_to_yaml_value(value)))
      .collect();

    Ok(TdrBuiltResource {
      schema: SchemaId::new(exported.schema.as_str()),
      header,
      content: exported.content,
    })
  }

  async fn list_schemas_impl(&self) -> RpcResult<Vec<SchemaId>> {
    let analysis = self.host.read().await.snapshot();
    let db = &analysis.db;
    let project = analysis.project;

    let config = get_vault_config(db, project);
    let schema_dir = config.schema_dir(db);
    let files = project.files(db);

    let mut schemas = Vec::new();
    for (path, file) in &files {
      if !path.starts_with(&schema_dir) {
        continue;
      }
      let Some(symbol) = file_symbol(db, project, *file).value(db) else {
        continue;
      };
      if !matches!(symbol.kind(db), SymbolKind::UserDefinedSchema(..)) {
        continue;
      }
      schemas.push(SchemaId::new(symbol.name(db)));
    }

    Ok(schemas)
  }

  async fn get_schema_impl(&self, schema: &SchemaId) -> RpcResult<TdrSchemaInfo> {
    let analysis = self.host.read().await.snapshot();
    let db = &analysis.db;
    let project = analysis.project;

    let config = get_vault_config(db, project);
    let schema_dir = config.schema_dir(db);
    let schema_path = schema_dir.join(format!("{schema}.tdr"));

    let files = project.files(db);
    let file = files.get(&schema_path).ok_or_else(|| {
      ErrorObjectOwned::owned(INVALID_PARAMS_CODE, "Schema not found", None::<()>)
    })?;

    let _symbol = file_symbol(db, project, *file).value(db).ok_or_else(|| {
      ErrorObjectOwned::owned(INVALID_PARAMS_CODE, "Schema has no symbol", None::<()>)
    })?;

    // TODO: Extract property details
    Ok(TdrSchemaInfo {
      schema: schema.clone(),
      properties: HashMap::new(),
    })
  }
}

fn exported_to_yaml_value(value: ExportedValue) -> YamlValue {
  match value {
    ExportedValue::String(string) => YamlValue::String(string),
    ExportedValue::Number(num) => YamlValue::Number(num),
    ExportedValue::Bool(boolean) => YamlValue::Bool(boolean),
    ExportedValue::List(items) => {
      YamlValue::List(items.into_iter().map(exported_to_yaml_value).collect())
    }
    ExportedValue::Object(map) => YamlValue::Object(
      map
        .into_iter()
        .map(|(key, val)| (YamlKeyId::new(key), exported_to_yaml_value(val)))
        .collect(),
    ),
    ExportedValue::Null => YamlValue::Null,
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

  async fn list_schemas(&self) -> RpcResult<Vec<SchemaId>> {
    self.list_schemas_impl().await
  }

  async fn get_schema(&self, schema: SchemaId) -> RpcResult<TdrSchemaInfo> {
    self.get_schema_impl(&schema).await
  }

  async fn on_content_changed(
    &self,
    pending: PendingSubscriptionSink,
  ) -> TdrRpcSubscriptionCloseResponse {
    run_subscription(pending, self.content_changed_tx.subscribe()).await
  }

  async fn on_content_created(
    &self,
    pending: PendingSubscriptionSink,
  ) -> TdrRpcSubscriptionCloseResponse {
    run_subscription(pending, self.content_created_tx.subscribe()).await
  }

  async fn on_content_deleted(
    &self,
    pending: PendingSubscriptionSink,
  ) -> TdrRpcSubscriptionCloseResponse {
    run_subscription(pending, self.content_deleted_tx.subscribe()).await
  }

  async fn on_schema_changed(
    &self,
    pending: PendingSubscriptionSink,
  ) -> TdrRpcSubscriptionCloseResponse {
    run_subscription(pending, self.schema_changed_tx.subscribe()).await
  }

  async fn on_schema_created(
    &self,
    pending: PendingSubscriptionSink,
  ) -> TdrRpcSubscriptionCloseResponse {
    run_subscription(pending, self.schema_created_tx.subscribe()).await
  }

  async fn on_schema_deleted(
    &self,
    pending: PendingSubscriptionSink,
  ) -> TdrRpcSubscriptionCloseResponse {
    run_subscription(pending, self.schema_deleted_tx.subscribe()).await
  }
}
