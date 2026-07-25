// core and lsp use OS-level APIs that don't exist in WASM
// These are used in server only btw
// We only need wasm for client
#[cfg(not(target_arch = "wasm32"))]
pub mod core;
#[cfg(not(target_arch = "wasm32"))]
pub mod lsp;

// client trait is used by both native and WASM
pub mod rpc;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
