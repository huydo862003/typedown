use std::collections::HashMap;
use std::path::{Path, PathBuf};

use lsp_types::{
  DocumentChangeOperation, DocumentChanges, OptionalVersionedTextDocumentIdentifier, RenameFile,
  ResourceOp, TextDocumentEdit, TextEdit, WorkspaceEdit,
};
use ropey::Rope;
use tdr_incremental::QueryDatabase;
use tdr_lang::db::derived::name_resolver::resolution_index::{Reference, ReferenceKind};
use tdr_lang::db::types::{HirValueKind, Symbol, SymbolKind};
use tdr_lang::db::{
  TypedownDatabase,
  derived::parse_file::parse_file,
  types::{File, Project},
};
use tdr_lang::syntax::ast::{AstNode, IdentLit};
use tdr_lang::syntax::red::RedNode;
use tdr_lang::syntax::syntax_kind::SyntaxKind;
use tdr_types::path::normalize_path;

use crate::analysis::Analysis;
use crate::utils::ast::{containing_fref_expr, find_ancestor, node_at_offset};
use crate::utils::position::text_offset_to_lsp_position;
use crate::utils::uri::path_to_uri;

use super::types::RenameSymbol;

/// Find the string content node (DqStrContent or SqStrContent) inside a StrLit
pub fn str_content_node(str_lit: &RedNode) -> Option<RedNode> {
  str_lit.children().find(|c| {
    matches!(
      c.kind(),
      SyntaxKind::DqStrContent | SyntaxKind::SqStrContent
    )
  })
}

/// Find the renameable symbol at a given offset in a file
pub fn find_rename_symbol(
  db: &TypedownDatabase,
  project: Project,
  file: File,
  offset: usize,
) -> Option<RenameSymbol> {
  let root = parse_file(db, project, file).ast(db);
  let node = node_at_offset(root, offset)?;

  if let Some(call_expr) = containing_fref_expr(&node) {
    // Reject rename if the fref argument is an interpolated string
    let arg = call_expr.arg(0)?;
    if arg
      .syntax()
      .children()
      .any(|c| c.kind() == SyntaxKind::InterpFragment)
    {
      return None;
    }
    return Some(RenameSymbol::Fref {
      call_node: call_expr,
    });
  }

  find_ancestor(&node, SyntaxKind::IdentLit)
    .and_then(IdentLit::cast)
    .map(|ident_node| RenameSymbol::Identifier { ident_node })
}

/// Get the file path backing a user-defined symbol
pub fn symbol_file_path(db: &dyn QueryDatabase, symbol: Symbol) -> Option<PathBuf> {
  match symbol.kind(db) {
    SymbolKind::UserDefinedSchema(_, file) | SymbolKind::UserDefinedResource(_, file) => {
      file.handle(db).path().cloned()
    }
    _ => None,
  }
}

/// Convert a RedNode to an LSP range using its trimmed span (excluding trivia)
fn trimmed_lsp_range(rope: &Rope, node: &RedNode) -> lsp_types::Range {
  let (offset, len) = node.trimmed_range();
  lsp_types::Range {
    start: text_offset_to_lsp_position(rope, offset),
    end: text_offset_to_lsp_position(rope, offset + len),
  }
}

/// Build text edits for all references to a symbol.
/// Ident references get replaced with `new_stem`.
/// Fref references get their path argument replaced with the new relative path.
pub fn collect_reference_edits(
  analysis: &Analysis,
  refs: &[Reference],
  new_stem: &str,
  new_absolute: &Path,
  root_dir: &Path,
) -> Option<HashMap<PathBuf, Vec<TextEdit>>> {
  let db = &analysis.db;
  let mut edits: HashMap<PathBuf, Vec<TextEdit>> = HashMap::new();

  for r in refs {
    let ref_path = r.hir.file(db).handle(db).path()?.clone();
    let ref_rope = analysis.file_rope(&ref_path)?;
    let node = r.hir.node(db);

    let text_edit = match r.kind {
      ReferenceKind::Ident => TextEdit {
        range: trimmed_lsp_range(&ref_rope, &node),
        new_text: new_stem.to_string(),
      },
      ReferenceKind::Fref => {
        let HirValueKind::Call { args, .. } = r.hir.kind(db) else {
          continue;
        };
        let Some(arg) = args.first() else { continue };
        let arg_node = arg.node(db);
        // Skip interpolated string arguments
        if arg_node
          .children()
          .any(|c| c.kind() == SyntaxKind::InterpFragment)
        {
          continue;
        }
        let Some(content) = str_content_node(&arg_node) else {
          continue;
        };
        let new_relative = new_absolute.strip_prefix(root_dir).ok()?;
        TextEdit {
          range: trimmed_lsp_range(&ref_rope, &content),
          new_text: normalize_path(new_relative).to_string(),
        }
      }
    };

    edits.entry(ref_path).or_default().push(text_edit);
  }

  Some(edits)
}

/// Assemble text edits and file renames into a WorkspaceEdit.
/// Text edits are ordered before file renames (edits reference old URIs).
pub fn build_workspace_edit(
  analysis: &Analysis,
  edits_by_path: HashMap<PathBuf, Vec<TextEdit>>,
  file_renames: Vec<(PathBuf, PathBuf)>,
) -> Option<WorkspaceEdit> {
  let mut changes: Vec<DocumentChangeOperation> = Vec::new();

  for (file_path, edits) in edits_by_path {
    let scheme = analysis
      .scheme_map
      .get(&file_path)
      .map(|s| s.as_str())
      .unwrap_or("file");
    changes.push(DocumentChangeOperation::Edit(TextDocumentEdit {
      text_document: OptionalVersionedTextDocumentIdentifier {
        uri: path_to_uri(&file_path, scheme),
        version: None,
      },
      edits: edits.into_iter().map(lsp_types::OneOf::Left).collect(),
    }));
  }

  for (old_path, new_path) in &file_renames {
    if old_path == new_path {
      continue;
    }
    let scheme = analysis
      .scheme_map
      .get(old_path)
      .map(|s| s.as_str())
      .unwrap_or("file");
    changes.push(DocumentChangeOperation::Op(ResourceOp::Rename(
      RenameFile {
        old_uri: path_to_uri(old_path, scheme),
        new_uri: path_to_uri(new_path, scheme),
        options: None,
        annotation_id: None,
      },
    )));
  }

  if changes.is_empty() {
    return None;
  }

  // Text edits before file renames
  changes.sort_by_key(|op| match op {
    DocumentChangeOperation::Edit(_) => 0,
    DocumentChangeOperation::Op(_) => 1,
  });

  Some(WorkspaceEdit {
    changes: None,
    document_changes: Some(DocumentChanges::Operations(changes)),
    change_annotations: None,
  })
}
