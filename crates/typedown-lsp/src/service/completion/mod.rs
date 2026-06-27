pub mod config;
pub mod tdr;

use lsp_types::{CompletionParams, CompletionResponse};

use crate::analysis::Analysis;
use crate::utils::uri::uri_to_path;

pub fn completion(analysis: &Analysis, params: CompletionParams) -> Option<CompletionResponse> {
  let path = uri_to_path(&params.text_document_position.text_document.uri)?;

  if path
    .file_name()
    .is_some_and(|name| name == "typedown.yaml" || name == "typedown.yml")
  {
    return config::completion(analysis, params);
  }

  tdr::completion(analysis, params)
}
