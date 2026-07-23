use jsonrpsee;
use jsonrpsee::proc_macros::rpc;

/// According to the doc, this generates two traits:
/// - TdrBuildRpcClient: An extension trait that adds all the required methods to a type that implements Client or SubscriptionClient
/// - TdrBuildRpcServer: A trait mostly equivalent to the input with
///    + An additional method `into_rpc` that converts TdrBuildRpcServer implementors to an `RpcModule`
///    + For subscription methods:
///      An additional param inserted after `&self`: `subscription_sink: SubscriptionSink`.
///      Return type must implement `IntoSubscriptionCloseResponse`.
#[rpc(server, client, namespace = "tdr_build", namespace_separator = ".")]
pub trait TdrBuildRpc<Hash, StorageKey> {
  #[method(name = "build_project")]
  async fn build_project(&self);

  #[method(name = "build_file")]
  async fn build_file(&self);

  #[subscription(name = "subscribe_file_changed", item = ())]
  async fn on_file_changed(&self);

  #[subscription(name = "subscribe_file_created", item = ())]
  async fn on_file_created(&self);

  #[subscription(name = "subscribe_file_deleted", item = ())]
  async fn on_file_deleted(&self);

  #[subscription(name = "subscribe_file_renamed", item = ())]
  async fn on_file_renamed(&self);
}
