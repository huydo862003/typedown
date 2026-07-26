use std::cell::RefCell;
use std::future::Future;
use std::sync::Arc;

use futures::future::{AbortHandle, Abortable};
use jsonrpsee::wasm_client::{Client as WasmClient, WasmClientBuilder};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::rpc::contract::{
  TdrBuildRpcClient, TdrBuiltResource, TdrFilePath, TdrSchemaInfo, TdrSiteConfig,
};

fn rpc_err(err: impl std::fmt::Display) -> JsValue {
  JsValue::from_str(&err.to_string())
}

#[wasm_bindgen]
pub struct RpcClient {
  inner: Arc<WasmClient>,
  content_changed: RefCell<Option<AbortHandle>>,
  content_created: RefCell<Option<AbortHandle>>,
  content_deleted: RefCell<Option<AbortHandle>>,
  schema_changed: RefCell<Option<AbortHandle>>,
  schema_created: RefCell<Option<AbortHandle>>,
  schema_deleted: RefCell<Option<AbortHandle>>,
  disconnect: RefCell<Option<AbortHandle>>,
}

impl RpcClient {
  fn register<Fut>(&self, slot: &RefCell<Option<AbortHandle>>, fut: Fut)
  // slot holds at most one active task
  where
    Fut: Future<Output = ()> + 'static,
  {
    // Create a linked pair:
    // - handle is the cancel control
    // - reg is given to the future
    let (handle, reg) = AbortHandle::new_pair();
    // Swap the new handle into the slot, getting back whatever was there before
    if let Some(old) = slot.borrow_mut().replace(handle) {
      // A previous task was registered here: cancel it before spawning the new one
      old.abort();
    }
    spawn_local(async move {
      // Wrap fut so it stops when handle.abort() is called, then hand it to the JS event loop
      let _ = Abortable::new(fut, reg).await;
    });
  }
}

impl Drop for RpcClient {
  fn drop(&mut self) {
    for handle in [
      self.content_changed.get_mut().take(),
      self.content_created.get_mut().take(),
      self.content_deleted.get_mut().take(),
      self.schema_changed.get_mut().take(),
      self.schema_created.get_mut().take(),
      self.schema_deleted.get_mut().take(),
      self.disconnect.get_mut().take(),
    ]
    .into_iter()
    .flatten()
    {
      handle.abort();
    }
  }
}

#[wasm_bindgen]
impl RpcClient {
  #[allow(unused_variables)]
  #[wasm_bindgen(static_method_of = RpcClient)]
  pub async fn connect(addr: String, port: u16) -> Result<RpcClient, JsValue> {
    let url = format!("ws://{addr}:{port}");
    let inner = WasmClientBuilder::default()
      .build(&url)
      .await
      .map_err(rpc_err)?;
    Ok(RpcClient {
      inner: Arc::new(inner),
      content_changed: RefCell::new(None),
      content_created: RefCell::new(None),
      content_deleted: RefCell::new(None),
      schema_changed: RefCell::new(None),
      schema_created: RefCell::new(None),
      schema_deleted: RefCell::new(None),
      disconnect: RefCell::new(None),
    })
  }

  pub async fn request_file(&self, path: String) -> Result<TdrBuiltResource, JsValue> {
    <WasmClient as TdrBuildRpcClient<(), ()>>::request_file(&*self.inner, TdrFilePath(path))
      .await
      .map_err(rpc_err)
  }

  pub async fn request_files(&self, paths: Vec<String>) -> Result<Vec<TdrBuiltResource>, JsValue> {
    let file_paths = paths.into_iter().map(TdrFilePath).collect();
    <WasmClient as TdrBuildRpcClient<(), ()>>::request_files(&*self.inner, file_paths)
      .await
      .map_err(rpc_err)
  }

  pub async fn list_vault(&self) -> Result<Vec<String>, JsValue> {
    <WasmClient as TdrBuildRpcClient<(), ()>>::list_vault(&*self.inner)
      .await
      .map_err(rpc_err)
  }

  pub async fn get_config(&self) -> Result<TdrSiteConfig, JsValue> {
    <WasmClient as TdrBuildRpcClient<(), ()>>::get_config(&*self.inner)
      .await
      .map_err(rpc_err)
  }

  pub async fn list_schemas(&self) -> Result<Vec<String>, JsValue> {
    <WasmClient as TdrBuildRpcClient<(), ()>>::list_schemas(&*self.inner)
      .await
      .map_err(rpc_err)
  }

  pub async fn get_schema(&self, schema: String) -> Result<TdrSchemaInfo, JsValue> {
    <WasmClient as TdrBuildRpcClient<(), ()>>::get_schema(&*self.inner, schema)
      .await
      .map_err(rpc_err)
  }

  pub fn on_content_changed(&self, callback: js_sys::Function) {
    let client = Arc::clone(&self.inner);
    self.register(&self.content_changed, async move {
      let Ok(mut sub) =
        <WasmClient as TdrBuildRpcClient<(), ()>>::subscribe_content_changed(&*client).await
      else {
        return;
      };
      while let Some(Ok(notif)) = sub.next().await {
        let _ = callback.call1(&JsValue::NULL, &notif.into());
      }
    });
  }

  pub fn on_content_created(&self, callback: js_sys::Function) {
    let client = Arc::clone(&self.inner);
    self.register(&self.content_created, async move {
      let Ok(mut sub) =
        <WasmClient as TdrBuildRpcClient<(), ()>>::subscribe_content_created(&*client).await
      else {
        return;
      };
      while let Some(Ok(notif)) = sub.next().await {
        let _ = callback.call1(&JsValue::NULL, &notif.into());
      }
    });
  }

  pub fn on_content_deleted(&self, callback: js_sys::Function) {
    let client = Arc::clone(&self.inner);
    self.register(&self.content_deleted, async move {
      let Ok(mut sub) =
        <WasmClient as TdrBuildRpcClient<(), ()>>::subscribe_content_deleted(&*client).await
      else {
        return;
      };
      while let Some(Ok(notif)) = sub.next().await {
        let _ = callback.call1(&JsValue::NULL, &notif.into());
      }
    });
  }

  pub fn on_schema_changed(&self, callback: js_sys::Function) {
    let client = Arc::clone(&self.inner);
    self.register(&self.schema_changed, async move {
      let Ok(mut sub) =
        <WasmClient as TdrBuildRpcClient<(), ()>>::subscribe_schema_changed(&*client).await
      else {
        return;
      };
      while let Some(Ok(notif)) = sub.next().await {
        let _ = callback.call1(&JsValue::NULL, &notif.into());
      }
    });
  }

  pub fn on_schema_created(&self, callback: js_sys::Function) {
    let client = Arc::clone(&self.inner);
    self.register(&self.schema_created, async move {
      let Ok(mut sub) =
        <WasmClient as TdrBuildRpcClient<(), ()>>::subscribe_schema_created(&*client).await
      else {
        return;
      };
      while let Some(Ok(notif)) = sub.next().await {
        let _ = callback.call1(&JsValue::NULL, &notif.into());
      }
    });
  }

  pub fn on_schema_deleted(&self, callback: js_sys::Function) {
    let client = Arc::clone(&self.inner);
    self.register(&self.schema_deleted, async move {
      let Ok(mut sub) =
        <WasmClient as TdrBuildRpcClient<(), ()>>::subscribe_schema_deleted(&*client).await
      else {
        return;
      };
      while let Some(Ok(notif)) = sub.next().await {
        let _ = callback.call1(&JsValue::NULL, &notif.into());
      }
    });
  }

  pub fn on_disconnect(&self, callback: js_sys::Function) {
    let client = Arc::clone(&self.inner);
    self.register(&self.disconnect, async move {
      client.on_disconnect().await;
      let _ = callback.call0(&JsValue::NULL);
    });
  }
}
