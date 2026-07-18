pub mod completion;
pub mod definition;
pub mod hover;
pub mod rename;
pub mod semantic_tokens;

use lsp_server::{ErrorCode, Request, Response};
use lsp_types::request::{
  Completion, GotoDefinition, HoverRequest, Request as _, SemanticTokensFullRequest,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::analysis::Analysis;

/// Dispatch an LSP request to the appropriate service handler.
pub fn dispatch(analysis: &Analysis, req: Request) -> Response {
  match req.method.as_str() {
    HoverRequest::METHOD => try_handle(&req, |p| hover::hover(analysis, p)),
    Completion::METHOD => try_handle(&req, |p| completion::completion(analysis, p)),
    GotoDefinition::METHOD => try_handle(&req, |p| definition::definition(analysis, p)),
    SemanticTokensFullRequest::METHOD => {
      try_handle(&req, |p| semantic_tokens::semantic_tokens_full(analysis, p))
    }
    _ => Response::new_err(
      req.id,
      ErrorCode::MethodNotFound as i32,
      format!("unhandled method: {}", req.method),
    ),
  }
}

// Deserialize params and call the handler
// Returns null on deserialization failure so the client always gets a valid reply
fn try_handle<P: DeserializeOwned, R: Serialize>(
  req: &Request,
  handler: impl FnOnce(P) -> Option<R>,
) -> Response {
  match serde_json::from_value::<P>(req.params.clone()) {
    Ok(params) => Response::new_ok(req.id.clone(), handler(params)),
    Err(err) => {
      log::warn!("Failed to deserialize {} params: {err}", req.method);
      Response::new_ok(req.id.clone(), Value::Null)
    }
  }
}
