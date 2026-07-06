//! Shared utilities for derived queries

use crate::syntax::ast::{AstNode, SourceFile};
use crate::syntax::diagnostic::Diagnostic;

use crate::db::TypedownDatabase;
use crate::db::derived::hir::lower_node;
use crate::db::derived::parse_file::parse_file;
use crate::db::types::{File, HirValue, Project};

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
