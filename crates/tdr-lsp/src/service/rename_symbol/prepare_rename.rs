use lsp_types::{PrepareRenameResponse, TextDocumentPositionParams};

use crate::{
  analysis::Analysis,
  service::rename_symbol::utils::find_rename_symbol,
  utils::{position::lsp_position_to_text_offset, uri::uri_to_path},
};

pub fn prepare_rename(
  analysis: &Analysis,
  params: TextDocumentPositionParams,
) -> Option<PrepareRenameResponse> {
  let project = analysis.project;

  // Locate the file of the rename request
  let path = uri_to_path(&params.text_document.uri)?;
  let file = *project.files(&analysis.db).get(&path)?;
  let rope = analysis.file_rope(&path)?;

  // Locate the offset of the rename request
  let editor_pos = params.position;
  let offset = lsp_position_to_text_offset(&rope, editor_pos)?;

  // Find the symbol that is requested a rename + qualifying information
  let rename_symbol = find_rename_symbol(&analysis.db, project, file, offset)?;

  Some(PrepareRenameResponse::Range(rename_symbol.get_range(&rope)))
}
