use lsp_types::{PrepareRenameResponse, TextDocumentPositionParams};

use crate::analysis::Analysis;

pub fn prepare_rename(
  analysis: &Analysis,
  params: TextDocumentPositionParams,
) -> Option<PrepareRenameResponse> {
  todo!()
}
