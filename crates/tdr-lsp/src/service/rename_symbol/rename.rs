use std::collections::HashMap;
use std::path::PathBuf;

use lsp_types::{
  DocumentChangeOperation, DocumentChanges, OptionalVersionedTextDocumentIdentifier, RenameFile,
  RenameParams, ResourceOp, TextDocumentEdit, TextEdit, WorkspaceEdit,
};
use tdr_lang::db::derived::hir::lower_node;
use tdr_lang::db::derived::name_resolver::referee::referee;
use tdr_lang::db::derived::name_resolver::resolution_index::{ReferenceKind, references};
use tdr_lang::db::types::{HirValueKind, SymbolKind};
use tdr_lang::syntax::ast::AstNode;

use crate::{
  analysis::Analysis,
  service::rename_symbol::{types::RenameSymbol, utils::find_rename_symbol},
  utils::{
    position::{lsp_position_to_text_offset, text_offset_to_lsp_position},
    uri::{path_to_uri, uri_to_path},
  },
};

pub fn rename(analysis: &Analysis, params: RenameParams) -> Option<WorkspaceEdit> {
  let db = &analysis.db;
  let project = analysis.project;
  let new_name = &params.new_name;

  // Locate the file and offset of the rename request
  let path = uri_to_path(&params.text_document_position.text_document.uri)?;
  let file = *project.files(db).get(&path)?;
  let rope = analysis.file_rope(&path)?;
  let offset = lsp_position_to_text_offset(&rope, params.text_document_position.position)?;

  // Find the symbol at the cursor
  let rename_symbol = find_rename_symbol(db, project, file, offset)?;

  let original_symbol = match rename_symbol {
    RenameSymbol::Fref { call_node } => referee(
      db,
      lower_node(db, project, file, call_node.syntax().clone()),
    ),
    RenameSymbol::Identifier { ident_node } => referee(
      db,
      lower_node(db, project, file, ident_node.syntax().clone()),
    ),
  }
  .value(db)?;

  let refs = references(db, project, original_symbol);
  if refs.is_empty() {
    return None;
  }

  let root_dir = project.root_dir(db);
  let mut changes: Vec<DocumentChangeOperation> = vec![];
  let mut edits_by_path: HashMap<PathBuf, Vec<TextEdit>> = HashMap::new();
  let mut file_rename_added = false;

  for r in &refs {
    let ref_file = r.hir.file(db);
    let ref_path = ref_file.handle(db).path()?.clone();
    let ref_rope = analysis.file_rope(&ref_path)?;
    let node = r.hir.node(db);

    match r.kind {
      ReferenceKind::Ident => {
        let start = text_offset_to_lsp_position(&ref_rope, node.offset());
        let end = text_offset_to_lsp_position(&ref_rope, node.offset() + node.text_len());
        edits_by_path.entry(ref_path).or_default().push(TextEdit {
          range: lsp_types::Range { start, end },
          new_text: new_name.clone(),
        });
      }
      ReferenceKind::Fref => {
        // The fref target file needs renaming
        // Compute old and new paths relative to the vault root
        let target_file_handle = original_symbol.kind(db);
        let target_file = match target_file_handle {
          SymbolKind::UserDefinedResource(_, file) | SymbolKind::UserDefinedSchema(_, file) => file,
          _ => continue,
        };
        let old_path = target_file.handle(db).path()?.clone();
        let old_relative = old_path.strip_prefix(&root_dir).ok()?;

        // New path: same directory, new file stem, same extension
        let parent = old_relative.parent().unwrap_or(std::path::Path::new(""));
        let extension = old_relative
          .extension()
          .and_then(|e| e.to_str())
          .unwrap_or("tdr");
        let new_relative = parent.join(format!("{}.{}", new_name, extension));
        let new_absolute = root_dir.join(&new_relative);

        // Validate: new path must stay within the same directory
        if new_absolute.parent() != old_path.parent() {
          continue;
        }

        // Add file rename operation (only once)
        if !file_rename_added {
          let scheme = analysis
            .scheme_map
            .get(&old_path)
            .map(|s| s.as_str())
            .unwrap_or("file");
          let old_uri = path_to_uri(&old_path, scheme);
          let new_uri = path_to_uri(&new_absolute, scheme);
          changes.push(DocumentChangeOperation::Op(ResourceOp::Rename(
            RenameFile {
              old_uri,
              new_uri,
              options: None,
              annotation_id: None,
            },
          )));
          file_rename_added = true;
        }

        // Update the fref string argument
        // Find the string arg inside the call node
        if let HirValueKind::Call { args, .. } = r.hir.kind(db)
          && let Some(arg) = args.first()
        {
          let arg_node = arg.node(db);
          let start = text_offset_to_lsp_position(&ref_rope, arg_node.offset());
          let end = text_offset_to_lsp_position(&ref_rope, arg_node.offset() + arg_node.text_len());
          let new_fref_path = new_relative.to_string_lossy();
          edits_by_path.entry(ref_path).or_default().push(TextEdit {
            range: lsp_types::Range { start, end },
            new_text: format!("\"{}\"", new_fref_path),
          });
        }
      }
    }
  }

  // Convert text edits to DocumentChangeOperations
  for (file_path, edits) in edits_by_path {
    let scheme = analysis
      .scheme_map
      .get(&file_path)
      .map(|s| s.as_str())
      .unwrap_or("file");
    let uri = path_to_uri(&file_path, scheme);
    changes.push(DocumentChangeOperation::Edit(TextDocumentEdit {
      text_document: OptionalVersionedTextDocumentIdentifier { uri, version: None },
      edits: edits.into_iter().map(lsp_types::OneOf::Left).collect(),
    }));
  }

  Some(WorkspaceEdit {
    changes: None,
    document_changes: Some(DocumentChanges::Operations(changes)),
    change_annotations: None,
  })
}
