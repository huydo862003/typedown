use lsp_types::{RenameParams, WorkspaceEdit};

use crate::analysis::Analysis;

pub fn rename(_analysis: &Analysis, _params: RenameParams) -> Option<WorkspaceEdit> {
  todo!()
}
