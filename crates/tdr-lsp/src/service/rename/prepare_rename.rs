use lsp_types::{PrepareRenameResponse, TextDocumentPositionParams};

use crate::analysis::Analysis;

pub fn prepare_rename(
  _analysis: &Analysis,
  _params: TextDocumentPositionParams,
) -> Option<PrepareRenameResponse> {
  todo!()
}
