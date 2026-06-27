use lsp_types::{CompletionParams, CompletionResponse};

use crate::analysis::Analysis;

pub fn completion(_analysis: &Analysis, _params: CompletionParams) -> Option<CompletionResponse> {
  None
}
