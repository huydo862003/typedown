use lsp_types::{DocumentChangeOperation, DocumentChanges, RenameParams, WorkspaceEdit};
use tdr_lang::{
  db::derived::{
    hir::lower_node,
    name_resolver::{referee::referee, resolution_index::references},
  },
  syntax::ast::AstNode,
};

use crate::{
  analysis::Analysis,
  service::rename::{types::RenameSymbol, utils::find_rename_symbol},
  utils::{position::lsp_position_to_text_offset, uri::uri_to_path},
};

pub fn rename(analysis: &Analysis, params: RenameParams) -> Option<WorkspaceEdit> {
  let project = analysis.project;

  // Locate the file of the rename request
  let path = uri_to_path(&params.text_document_position.text_document.uri)?;
  let file = *project.files(&analysis.db).get(&path)?;
  let rope = analysis.file_rope(&path)?;

  // Locate the offset of the rename request
  let editor_pos = params.text_document_position.position;
  let offset = lsp_position_to_text_offset(&rope, editor_pos)?;

  // Find the symbol that is requested a rename + qualifying information
  let rename_symbol = find_rename_symbol(&analysis.db, project, file, offset)?;

  let original_symbol = match rename_symbol {
    RenameSymbol::Fref { call_node } => referee(
      &analysis.db,
      lower_node(&analysis.db, project, file, call_node.syntax().clone()),
    ),
    RenameSymbol::Identifier { ident_node } => referee(
      &analysis.db,
      lower_node(&analysis.db, project, file, ident_node.syntax().clone()),
    ),
  }
  .value(&analysis.db)?;

  let references = references(&analysis.db, project, original_symbol).references(&analysis.db);

  if references.len() == 0 {
    return None;
  }

  let changes: Vec<DocumentChangeOperation> = vec![];

  for reference in references {

  }

  Some(WorkspaceEdit {
    changes: None,
    document_changes: Some(DocumentChanges::Operations(changes)),
    change_annotations: None,
  })
}
