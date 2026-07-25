use std::path::{Path, PathBuf};
use std::sync::Arc;

use jsonrpsee::server::Server;
use tdr_server::rpc::contract::TdrBuildRpcServer;
use tdr_server::rpc::server::RpcServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let start = std::env::var("TDR_RPC_ROOT")
    .map(PathBuf::from)
    .unwrap_or_else(|_| std::env::current_dir().expect("failed to get current directory"));

  let root_dir = find_vault_root(&start)?;

  let addr = std::env::var("TDR_RPC_ADDR").unwrap_or_else(|_| "127.0.0.1".to_string());
  let port = std::env::var("TDR_RPC_PORT").unwrap_or_else(|_| "0".to_string());

  let rpc_server = RpcServer::new(root_dir)?;
  let module = Arc::try_unwrap(rpc_server)
    .ok()
    .expect("no other Arc references at startup")
    .into_rpc();

  let server = Server::builder().build(format!("{addr}:{port}")).await?;
  let addr = server.local_addr()?;
  let handle = server.start(module);

  println!("ws://{addr}");

  handle.stopped().await;

  Ok(())
}

fn find_vault_root(start: &Path) -> anyhow::Result<PathBuf> {
  let mut current = start.to_path_buf();
  loop {
    if current.join("typedown.yaml").exists() || current.join("typedown.yml").exists() {
      return Ok(current);
    }
    if !current.pop() {
      anyhow::bail!(
        "no typedown.yaml or typedown.yml found in {} or any parent",
        start.display()
      );
    }
  }
}
