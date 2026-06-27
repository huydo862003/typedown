use lsp_types::{Hover, HoverParams};

use crate::analysis::Analysis;

pub fn hover(_analysis: &Analysis, _params: HoverParams) -> Option<Hover> {
  None
}
