//! Shared utilities for derived queries

use typedown_syntax::ast::{AstNode, SourceFile};
use typedown_types::diagnostic::Diagnostic;

use crate::TypedownDatabase;
use crate::derived::hir::lower_expr;
use crate::derived::parse_file::parse_file;
use crate::types::{File, HirValue, Project};

/// Parse a file and lower its frontmatter mapping to HIR.
pub fn lower_frontmatter(
  db: &TypedownDatabase,
  project: Project,
  file: File,
) -> (Option<HirValue>, Vec<Diagnostic>) {
  let parse_result = parse_file(db, project, file);
  let diagnostics = parse_result.diagnostics(db).to_vec();
  let root = parse_result.ast(db);
  let source_file = match SourceFile::cast(root) {
    Some(sf) => sf,
    None => return (None, diagnostics),
  };
  let mapping = match source_file.frontmatter().and_then(|fm| fm.mapping()) {
    Some(m) => m,
    None => return (None, diagnostics),
  };
  let hir = lower_expr(db, project, file, mapping.syntax().clone());
  (Some(hir), diagnostics)
}
