//! Shared utilities for derived queries

pub mod typecheck;

use crate::syntax::ast::{AstNode, SourceFile};
use crate::syntax::diagnostic::Diagnostic;
use crate::syntax::red::RedNode;
use crate::syntax::syntax_kind::SyntaxKind;

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

/// Find the value of the _type field in a mapping or dict node
pub fn schema_name_in_mapping(mapping: &RedNode) -> Option<String> {
  for entry in mapping.children() {
    // Block mapping entry
    if entry.kind() == SyntaxKind::YamlMappingEntry {
      let mut children = entry.children();
      let key = children.find(|child| child.kind() == SyntaxKind::YamlMappingEntryKey)?;
      if key.text().trim() != "_type" {
        continue;
      }
      let value = children.find(|child| child.kind() == SyntaxKind::YamlMappingEntryValue)?;
      return Some(value.text().trim().to_string());
    }
    // Flow dict entry
    if entry.kind() == SyntaxKind::DictEntry {
      let mut children = entry.children();
      let key = children.find(|child| child.kind() == SyntaxKind::DictEntryKey)?;
      if key.text().trim() != "_type" {
        continue;
      }
      let value = children.find(|child| child.kind() == SyntaxKind::DictEntryValue)?;
      return Some(value.text().trim().to_string());
    }
  }
  None
}
