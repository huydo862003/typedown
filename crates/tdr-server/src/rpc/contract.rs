#[cfg(not(target_arch = "wasm32"))]
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

#[cfg(not(target_arch = "wasm32"))]
use jsonrpsee::{
  IntoSubscriptionCloseResponse, SubscriptionCloseResponse, core::to_json_raw_value,
};

/// On native: generates both TdrBuildRpcServer and TdrBuildRpcClient traits
#[cfg_attr(
  not(target_arch = "wasm32"),
  rpc(server, client, namespace = "tdr_build", namespace_separator = ".")
)]
/// On WASM: generates only TdrBuildRpcClient (no server types available)
#[cfg_attr(
  target_arch = "wasm32",
  rpc(client, namespace = "tdr_build", namespace_separator = ".")
)]
pub trait TdrBuildRpc<Hash, StorageKey> {
  /* Requests */

  #[method(name = "request_file")]
  async fn request_file(&self, file_path: TdrFilePath) -> RpcResult<TdrBuiltResource>;

  #[method(name = "request_files")]
  async fn request_files(&self, file_paths: Vec<TdrFilePath>) -> RpcResult<Vec<TdrBuiltResource>>;

  #[method(name = "list_vault")]
  async fn list_vault(&self) -> RpcResult<Vec<String>>;

  #[method(name = "list_schemas")]
  async fn list_schemas(&self) -> RpcResult<Vec<String>>;

  #[method(name = "get_schema")]
  async fn get_schema(&self, schema: String) -> RpcResult<TdrSchemaInfo>;

  #[method(name = "get_config")]
  async fn get_config(&self) -> RpcResult<TdrSiteConfig>;

  /* Content subscriptions */

  #[subscription(name = "subscribe_content_changed", item = TdrContentNotification)]
  async fn subscribe_content_changed(&self) -> TdrRpcSubscriptionCloseResponse;

  #[subscription(name = "subscribe_content_created", item = TdrContentNotification)]
  async fn subscribe_content_created(&self) -> TdrRpcSubscriptionCloseResponse;

  #[subscription(name = "subscribe_content_deleted", item = TdrContentNotification)]
  async fn subscribe_content_deleted(&self) -> TdrRpcSubscriptionCloseResponse;

  /* Schema subscriptions */

  #[subscription(name = "subscribe_schema_changed", item = TdrSchemaNotification)]
  async fn subscribe_schema_changed(&self) -> TdrRpcSubscriptionCloseResponse;

  #[subscription(name = "subscribe_schema_created", item = TdrSchemaNotification)]
  async fn subscribe_schema_created(&self) -> TdrRpcSubscriptionCloseResponse;

  #[subscription(name = "subscribe_schema_deleted", item = TdrSchemaNotification)]
  async fn subscribe_schema_deleted(&self) -> TdrRpcSubscriptionCloseResponse;
}

/* RPC request params and results */

/// Site-wide configuration derived from typedown.yaml
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[cfg_attr(target_arch = "wasm32", tsify(into_wasm_abi))]
pub struct TdrSiteConfig {
  /// URL base path (e.g. "/" or "/docs")
  pub base_path: String,
  /// Content directory path relative to the project root
  pub content_dir: String,
}

/// Path relative to the content directory
#[derive(Serialize, Deserialize)]
pub struct TdrFilePath(pub String);

/// Structured build result: Header (frontmatter) and content (commonmark body)
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[cfg_attr(target_arch = "wasm32", tsify(into_wasm_abi))]
pub struct TdrBuiltResource {
  pub schema: String,
  #[cfg_attr(target_arch = "wasm32", tsify(type = "Record<string, any>"))]
  pub header: serde_json::Value,
  pub content: String,
}

/// Schema metadata
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[cfg_attr(target_arch = "wasm32", tsify(into_wasm_abi))]
pub struct TdrSchemaInfo {
  pub schema: String,
  #[cfg_attr(target_arch = "wasm32", tsify(type = "Record<string, any>"))]
  pub properties: serde_json::Value,
}

/* Subscription notifications */

/// Content file event: A resource file was created, changed, or deleted
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[cfg_attr(target_arch = "wasm32", tsify(into_wasm_abi))]
pub struct TdrContentNotification {
  pub content: String,
}

/// Schema file event: A schema file was created, changed, or deleted
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[cfg_attr(target_arch = "wasm32", tsify(into_wasm_abi))]
pub struct TdrSchemaNotification {
  pub schema: String,
}

/* Server's response to client subscription termination */

#[cfg(not(target_arch = "wasm32"))]
pub enum TdrRpcSubscriptionCloseResponse {
  Ok,
  Err(String),
}

#[cfg(not(target_arch = "wasm32"))]
impl IntoSubscriptionCloseResponse for TdrRpcSubscriptionCloseResponse {
  fn into_response(self) -> SubscriptionCloseResponse {
    match self {
      TdrRpcSubscriptionCloseResponse::Ok => SubscriptionCloseResponse::None,
      TdrRpcSubscriptionCloseResponse::Err(msg) => {
        let err = to_json_raw_value(&msg).unwrap();
        SubscriptionCloseResponse::Notif(err.into())
      }
    }
  }
}
