pub mod completion;
pub mod definition;
pub mod hover;
pub mod semantic_tokens;

use lsp_server::{ErrorCode, Request, Response};
use lsp_types::request::{
  Completion, GotoDefinition, HoverRequest, Request as _, SemanticTokensFullRequest,
};

use crate::analysis::Analysis;

/// Dispatch an LSP request to the appropriate service handler.
pub fn dispatch(analysis: &Analysis, req: Request) -> Response {
  match req.method.as_str() {
    HoverRequest::METHOD => {
      let params = serde_json::from_value(req.params).ok();
      let result = params.and_then(|p| hover::hover(analysis, p));
      Response::new_ok(req.id, result)
    }
    Completion::METHOD => {
      let params = serde_json::from_value(req.params).ok();
      let result = params.and_then(|p| completion::completion(analysis, p));
      Response::new_ok(req.id, result)
    }
    GotoDefinition::METHOD => {
      let params = serde_json::from_value(req.params).ok();
      let result = params.and_then(|p| definition::definition(analysis, p));
      Response::new_ok(req.id, result)
    }
    SemanticTokensFullRequest::METHOD => {
      let params = serde_json::from_value(req.params).ok();
      let result = params.and_then(|p| semantic_tokens::semantic_tokens_full(analysis, p));
      Response::new_ok(req.id, result)
    }
    _ => Response::new_err(
      req.id,
      ErrorCode::MethodNotFound as i32,
      format!("unhandled method: {}", req.method),
    ),
  }
}
