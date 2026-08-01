use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use jsonrpsee::core::{RpcResult, async_trait};
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::types::error::INVALID_PARAMS_CODE;
use jsonrpsee::{PendingSubscriptionSink, SubscriptionMessage, SubscriptionSink};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use tokio::sync::broadcast;
use tokio::time::{Duration, sleep};
use typedown_incremental::QueryStorage;
use typedown_lang::db::TypedownDatabase;
use typedown_lang::db::derived::get_vault_config::get_vault_config;
use typedown_lang::db::derived::name_resolver::file_symbol::file_symbol;
use typedown_lang::db::types::{AssetsDirMode, SymbolKind};
use typedown_lang::integrations::export::{export_resource, export_schema};

use typedown_types::path::normalize_path;

use crate::core::analysis_host::AnalysisHost;
use crate::core::utils::fs::{is_asset_file, is_td_file, is_vault_config};

use super::contract::{
  TdAssetsDir, TdBuildRpcServer, TdBuiltResource, TdContentNotification, TdContentSummary,
  TdFilePath, TdRpcSubscriptionCloseResponse, TdSchemaInfo, TdSchemaNotification, TdSiteConfig,
};

enum FsEventKind {
  Created,
  Modified,
  Removed,
}

struct FsEvent {
  path: PathBuf,
  kind: FsEventKind,
}

// Build a TdSiteConfig from the current project state
fn build_site_config(
  db: &TypedownDatabase,
  project: typedown_lang::db::types::Project,
) -> TdSiteConfig {
  let config = get_vault_config(db, project);
  let root = project.root_dir(db);
  let content_dir = config.content_dir(db);
  let base_path = config.base_path(db);
  let assets_dir = config.assets_dir(db);

  let content_dir_rel = normalize_path(content_dir.strip_prefix(&root).unwrap_or(&content_dir));

  TdSiteConfig {
    base_path: base_path.to_string(),
    content_dir: content_dir_rel,
    assets_dir: TdAssetsDir {
      mode: match assets_dir.mode {
        AssetsDirMode::Local => "local".to_string(),
      },
      path: assets_dir.path.clone(),
    },
    site_title: config.site_title(db).to_string(),
    site_description: config.site_description(db).to_string(),
  }
}

/// RPC build server that holds a single project and serves build requests
// TIL: Use tokio::sync::RwLock in async contexts, not std::sync::RwLock as std::sync::RwLock blocks the OS thread while waiting, which can deadlock the tokio runtime if the lock is held across an .await point
pub struct RpcServer {
  host: Arc<tokio::sync::RwLock<AnalysisHost>>,
  events: Arc<FsEventBus>,
}

// Shared state for the FS watcher task
struct FsEventBus {
  // Content events
  content_changed_tx: broadcast::Sender<TdContentNotification>,
  content_created_tx: broadcast::Sender<TdContentNotification>,
  content_deleted_tx: broadcast::Sender<TdContentNotification>,
  // Schema events
  schema_changed_tx: broadcast::Sender<TdSchemaNotification>,
  schema_created_tx: broadcast::Sender<TdSchemaNotification>,
  schema_deleted_tx: broadcast::Sender<TdSchemaNotification>,
  // Config events
  config_changed_tx: broadcast::Sender<TdSiteConfig>,
  // Held to keep the watcher alive for the lifetime of the server
  _watcher: RecommendedWatcher,
}

impl RpcServer {
  pub fn new(root_dir: PathBuf) -> anyhow::Result<Self> {
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
    let (config_changed_tx, _) = broadcast::channel(64);

    let (fs_tx, fs_rx) = tokio::sync::mpsc::unbounded_channel();
    let _watcher = Self::setup_watcher(&root_dir, fs_tx)?;

    let host = Arc::new(tokio::sync::RwLock::new(host));
    let events = Arc::new(FsEventBus {
      content_changed_tx,
      content_created_tx,
      content_deleted_tx,
      schema_changed_tx,
      schema_created_tx,
      schema_deleted_tx,
      config_changed_tx,
      _watcher,
    });

    Self::spawn_fs_watcher_task(Arc::clone(&host), Arc::clone(&events), fs_rx);

    Ok(Self { host, events })
  }

  /// Set up the file watcher
  fn setup_watcher(
    root_dir: &Path,
    fs_tx: tokio::sync::mpsc::UnboundedSender<FsEvent>,
  ) -> anyhow::Result<RecommendedWatcher> {
    let mut watcher = notify::recommended_watcher(move |result: Result<Event, notify::Error>| {
      let Ok(event) = result else { return };
      for path in &event.paths {
        if !is_td_file(path) && !is_asset_file(path) && !is_vault_config(path) {
          continue;
        }
        let kind = match event.kind {
          EventKind::Create(_) => FsEventKind::Created,
          EventKind::Modify(_) => FsEventKind::Modified,
          EventKind::Remove(_) => FsEventKind::Removed,
          _ => continue,
        };
        let _ = fs_tx.send(FsEvent {
          path: path.clone(),
          kind,
        });
      }
    })?;

    watcher.watch(root_dir, RecursiveMode::Recursive)?;
    Ok(watcher)
  }

  fn spawn_fs_watcher_task(
    host: Arc<tokio::sync::RwLock<AnalysisHost>>,
    events: Arc<FsEventBus>,
    mut fs_rx: tokio::sync::mpsc::UnboundedReceiver<FsEvent>,
  ) {
    tokio::spawn(async move {
      loop {
        let Some(first) = fs_rx.recv().await else {
          break;
        };
        let mut pending: HashMap<PathBuf, FsEvent> = HashMap::new();
        pending.insert(first.path.clone(), first);

        // Drain additional events within 50ms so rapid editor saves batch together
        let deadline = sleep(Duration::from_millis(50));
        tokio::pin!(deadline);
        loop {
          tokio::select! {
            event = fs_rx.recv() => match event {
              Some(ev) => { pending.insert(ev.path.clone(), ev); }
              None => return,
            },
            _ = &mut deadline => break,
          }
        }

        let mut host_guard = host.write().await;
        for event in pending.values() {
          match event.kind {
            FsEventKind::Created | FsEventKind::Modified => {
              host_guard.on_disk_change(event.path.clone())
            }
            FsEventKind::Removed => host_guard.on_disk_delete(event.path.clone()),
          }
        }
        let analysis = host_guard.snapshot();
        drop(host_guard);

        let db = &analysis.db;
        let project = analysis.project;
        let config = get_vault_config(db, project);
        let content_dir = config.content_dir(db);
        let schema_dir = config.schema_dir(db);

        // Notify subscribers if any pending event is a config file change
        if pending.values().any(|event| is_vault_config(&event.path)) {
          let _ = events
            .config_changed_tx
            .send(build_site_config(db, project));
        }

        for event in pending.into_values() {
          if event.path.starts_with(&content_dir) {
            let notification = TdContentNotification {
              content: event.path.to_string_lossy().into_owned(),
            };
            let sender = match event.kind {
              FsEventKind::Created => &events.content_created_tx,
              FsEventKind::Modified => &events.content_changed_tx,
              FsEventKind::Removed => &events.content_deleted_tx,
            };
            let _ = sender.send(notification);
          } else if event.path.starts_with(&schema_dir) {
            let Some(name) = event
              .path
              .file_stem()
              .and_then(|s| s.to_str())
              .map(str::to_string)
            else {
              continue;
            };
            let notification = TdSchemaNotification { schema: name };
            let sender = match event.kind {
              FsEventKind::Created => &events.schema_created_tx,
              FsEventKind::Modified => &events.schema_changed_tx,
              FsEventKind::Removed => &events.schema_deleted_tx,
            };
            let _ = sender.send(notification);
          }
        }
      }
    });
  }

  async fn build_file_impl(&self, file_path: &TdFilePath) -> RpcResult<TdBuiltResource> {
    let mut results = self
      .build_files_impl(std::slice::from_ref(file_path))
      .await?;
    Ok(results.swap_remove(0))
  }

  async fn build_files_impl(&self, file_paths: &[TdFilePath]) -> RpcResult<Vec<TdBuiltResource>> {
    let analysis = self.host.read().await.snapshot();
    let db = &analysis.db;
    let project = analysis.project;

    let config = get_vault_config(db, project);
    let content_dir = config.content_dir(db);
    let files = project.files(db);

    let mut results = Vec::with_capacity(file_paths.len());
    for file_path in file_paths {
      let path = content_dir.join(&file_path.0);
      let file = files.get(&path).ok_or_else(|| {
        ErrorObjectOwned::owned(
          INVALID_PARAMS_CODE,
          format!("File not found: {}", file_path.0),
          None::<()>,
        )
      })?;

      let exported = export_resource(db, project, *file).ok_or_else(|| {
        ErrorObjectOwned::owned(
          INVALID_PARAMS_CODE,
          format!("File is not a resource: {}", file_path.0),
          None::<()>,
        )
      })?;

      results.push(TdBuiltResource {
        schema: exported.schema,
        header: exported.header,
        content: exported.content,
      });
    }

    Ok(results)
  }

  async fn list_vault_impl(&self) -> RpcResult<Vec<String>> {
    let analysis = self.host.read().await.snapshot();
    let db = &analysis.db;
    let project = analysis.project;

    let config = get_vault_config(db, project);
    let content_dir = config.content_dir(db);
    let files = project.files(db);

    let mut result = Vec::new();
    for path in files.keys() {
      if !path.starts_with(&content_dir) {
        continue;
      }
      if path.extension().and_then(|e| e.to_str()) != Some("td") {
        continue;
      }
      let rel = path.strip_prefix(&content_dir).unwrap_or(path);
      result.push(normalize_path(rel));
    }

    Ok(result)
  }

  async fn list_files_grouped_by_schema_impl(
    &self,
  ) -> RpcResult<HashMap<String, Vec<TdContentSummary>>> {
    let analysis = self.host.read().await.snapshot();
    let db = &analysis.db;
    let project = analysis.project;

    let config = get_vault_config(db, project);
    let content_dir = config.content_dir(db);
    let files = project.files(db);

    let mut groups: HashMap<String, Vec<TdContentSummary>> = HashMap::new();
    for (path, file) in files.iter() {
      if !path.starts_with(&content_dir) {
        continue;
      }
      if path.extension().and_then(|e| e.to_str()) != Some("td") {
        continue;
      }
      let rel = normalize_path(path.strip_prefix(&content_dir).unwrap_or(path));
      if let Some(exported) = export_resource(db, project, *file) {
        groups
          .entry(exported.schema.clone())
          .or_default()
          .push(TdContentSummary {
            path: rel,
            schema: exported.schema,
            header: exported.header,
          });
      }
    }

    Ok(groups)
  }

  async fn get_config_impl(&self) -> RpcResult<TdSiteConfig> {
    let analysis = self.host.read().await.snapshot();
    let db = &analysis.db;
    let project = analysis.project;

    Ok(build_site_config(db, project))
  }

  async fn list_schemas_impl(&self) -> RpcResult<Vec<String>> {
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
      schemas.push(symbol.name(db).to_string());
    }

    Ok(schemas)
  }

  async fn get_schema_impl(&self, schema: &str) -> RpcResult<TdSchemaInfo> {
    let analysis = self.host.read().await.snapshot();
    let db = &analysis.db;
    let project = analysis.project;

    let config = get_vault_config(db, project);
    let schema_path = config.schema_dir(db).join(format!("{schema}.td"));

    let files = project.files(db);
    let file = files.get(&schema_path).ok_or_else(|| {
      ErrorObjectOwned::owned(INVALID_PARAMS_CODE, "Schema not found", None::<()>)
    })?;

    let properties = export_schema(db, project, *file)
      .unwrap_or_else(|| serde_json::Value::Object(Default::default()));

    Ok(TdSchemaInfo {
      schema: schema.to_string(),
      properties,
    })
  }
}

/// Accept a subscription and forward broadcast messages to the client
async fn run_subscription<T: Serialize + Clone>(
  pending: PendingSubscriptionSink,
  mut rx: broadcast::Receiver<T>,
) -> TdRpcSubscriptionCloseResponse {
  let Ok(sink) = pending.accept().await else {
    return TdRpcSubscriptionCloseResponse::Err("Failed to accept subscription".into());
  };
  while let Ok(notification) = rx.recv().await {
    if !forward(&sink, &notification).await {
      break;
    }
  }
  TdRpcSubscriptionCloseResponse::Ok
}

async fn forward<T: Serialize>(sink: &SubscriptionSink, value: &T) -> bool {
  let Ok(msg) = SubscriptionMessage::new(sink.method_name(), sink.subscription_id(), value) else {
    return true; // Skip unserializable notification, keep subscription alive
  };
  sink.send(msg).await.is_ok()
}

#[async_trait]
impl TdBuildRpcServer<(), ()> for RpcServer {
  async fn request_file(&self, file_path: TdFilePath) -> RpcResult<TdBuiltResource> {
    self.build_file_impl(&file_path).await
  }

  async fn request_files(&self, file_paths: Vec<TdFilePath>) -> RpcResult<Vec<TdBuiltResource>> {
    self.build_files_impl(&file_paths).await
  }

  async fn list_vault(&self) -> RpcResult<Vec<String>> {
    self.list_vault_impl().await
  }

  async fn list_files_grouped_by_schema(
    &self,
  ) -> RpcResult<HashMap<String, Vec<TdContentSummary>>> {
    self.list_files_grouped_by_schema_impl().await
  }

  async fn list_schemas(&self) -> RpcResult<Vec<String>> {
    self.list_schemas_impl().await
  }

  async fn get_schema(&self, schema: String) -> RpcResult<TdSchemaInfo> {
    self.get_schema_impl(&schema).await
  }

  async fn get_config(&self) -> RpcResult<TdSiteConfig> {
    self.get_config_impl().await
  }

  async fn subscribe_content_changed(
    &self,
    pending: PendingSubscriptionSink,
  ) -> TdRpcSubscriptionCloseResponse {
    run_subscription(pending, self.events.content_changed_tx.subscribe()).await
  }

  async fn subscribe_content_created(
    &self,
    pending: PendingSubscriptionSink,
  ) -> TdRpcSubscriptionCloseResponse {
    run_subscription(pending, self.events.content_created_tx.subscribe()).await
  }

  async fn subscribe_content_deleted(
    &self,
    pending: PendingSubscriptionSink,
  ) -> TdRpcSubscriptionCloseResponse {
    run_subscription(pending, self.events.content_deleted_tx.subscribe()).await
  }

  async fn subscribe_schema_changed(
    &self,
    pending: PendingSubscriptionSink,
  ) -> TdRpcSubscriptionCloseResponse {
    run_subscription(pending, self.events.schema_changed_tx.subscribe()).await
  }

  async fn subscribe_schema_created(
    &self,
    pending: PendingSubscriptionSink,
  ) -> TdRpcSubscriptionCloseResponse {
    run_subscription(pending, self.events.schema_created_tx.subscribe()).await
  }

  async fn subscribe_schema_deleted(
    &self,
    pending: PendingSubscriptionSink,
  ) -> TdRpcSubscriptionCloseResponse {
    run_subscription(pending, self.events.schema_deleted_tx.subscribe()).await
  }

  async fn subscribe_config_changed(
    &self,
    pending: PendingSubscriptionSink,
  ) -> TdRpcSubscriptionCloseResponse {
    run_subscription(pending, self.events.config_changed_tx.subscribe()).await
  }
}
