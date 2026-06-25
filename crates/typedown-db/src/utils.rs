//! Shared utilities for derived queries

use typedown_syntax::ast::{AstNode, SourceFile};
use typedown_types::diagnostic::Diagnostic;

use crate::TypedownDatabase;
use crate::derived::hir::lower_node;
use crate::derived::parse_file::parse_file;
use crate::types::{File, HirValue, Project};

pub fn lower_file(
  db: &TypedownDatabase,
  project: Project,
  file: File,
) -> (Option<HirValue>, Vec<Diagnostic>) {
  let parse_result = parse_file(db, project, file);
  let diagnostics = parse_result.diagnostics(db).to_vec();
  let root = parse_result.ast(db);
  if SourceFile::cast(root.clone()).is_none() {
    return (None, diagnostics);
  }
  let hir = lower_node(db, project, file, root);
  (Some(hir), diagnostics)
}
