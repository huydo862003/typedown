use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse};

use crate::analysis::Analysis;

pub fn definition(
  _analysis: &Analysis,
  _params: GotoDefinitionParams,
) -> Option<GotoDefinitionResponse> {
  None
}
