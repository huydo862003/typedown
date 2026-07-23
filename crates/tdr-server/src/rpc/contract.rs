use jsonrpsee::core::{RpcResult, to_json_raw_value};
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::{self, IntoSubscriptionCloseResponse, SubscriptionCloseResponse};
use serde::{Deserialize, Serialize};

/// According to the doc, this generates two traits:
/// - TdrBuildRpcClient: An extension trait that adds all the required methods to a type that implements Client or SubscriptionClient
/// - TdrBuildRpcServer: A trait mostly equivalent to the input with
///    + An additional method `into_rpc` that converts TdrBuildRpcServer implementors to an `RpcModule`
///    + For subscription methods:
///      An additional param inserted after `&self`: `subscription_sink: SubscriptionSink`.
///      Return type must implement `IntoSubscriptionCloseResponse`.
#[rpc(server, client, namespace = "tdr_build", namespace_separator = ".")]
pub trait TdrBuildRpc<Hash, StorageKey> {
  #[method(name = "request_file")]
  async fn request_file(
    &self,
    file_path: TdrFilePath,
    format: TdrBuildFormat,
  ) -> RpcResult<TdrBuiltResource>;

  #[subscription(name = "subscribe_file_changed", item = TdrFileChangedNotification)]
  async fn on_file_changed(&self) -> TdrRpcSubscriptionCloseResponse;

  #[subscription(name = "subscribe_file_created", item = TdrFileCreatedNotification)]
  async fn on_file_created(&self) -> TdrRpcSubscriptionCloseResponse;

  #[subscription(name = "subscribe_file_deleted", item = TdrFileDeletedNotification)]
  async fn on_file_deleted(&self) -> TdrRpcSubscriptionCloseResponse;

  #[subscription(name = "subscribe_file_renamed", item = TdrFileRenamedNotification)]
  async fn on_file_renamed(&self) -> TdrRpcSubscriptionCloseResponse;
}

/* RPC request params and results */

/// Must be absolute, the root is relative to the vault root
#[derive(Serialize, Deserialize)]
pub struct TdrFilePath(String);

#[derive(Serialize, Deserialize)]
pub enum TdrBuildFormat {
  Json,
  Markdown,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum TdrBuiltResource {
  Json {},
  Markdown {},
}

/* RPC subscription items */

#[derive(Deserialize)]
pub struct TdrFileChangedNotification {}

#[derive(Deserialize)]
pub struct TdrFileCreatedNotification {}

#[derive(Deserialize)]
pub struct TdrFileDeletedNotification {}

#[derive(Deserialize)]
pub struct TdrFileRenamedNotification {}

/* Server's response to client subscription termination */

pub enum TdrRpcSubscriptionCloseResponse {
  Ok,
  Err(String),
}

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
