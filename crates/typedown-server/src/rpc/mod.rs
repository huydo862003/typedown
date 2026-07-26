pub mod contract;

// server uses tokio, notify, and other OS-level APIs unavailable in WASM
#[cfg(not(target_arch = "wasm32"))]
pub mod server;
