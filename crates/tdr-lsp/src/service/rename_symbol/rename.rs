use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use lsp_types::{
  DocumentChangeOperation, DocumentChanges, OptionalVersionedTextDocumentIdentifier, RenameFile,
  RenameParams, ResourceOp, TextDocumentEdit, TextEdit, WorkspaceEdit,
};
use tdr_lang::db::derived::get_vault_config::get_vault_config;
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
  let mut file_rename_map = HashSet::<PathBuf>::new();

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
        let old_absolute = target_file.handle(db).path()?.clone();
        let old_relative = old_absolute.strip_prefix(&root_dir).ok()?;

        // New path: same directory, new file stem, same extension
        let parent = old_relative.parent().unwrap_or(Path::new(""));
        let extension = old_relative
          .extension()
          .and_then(|e| e.to_str())
          .unwrap_or("tdr");
        let new_relative = parent.join(format!(
          "{}.{}",
          new_name.strip_suffix(".tdr").unwrap_or(new_name),
          extension
        ));
        let new_absolute = root_dir.join(&new_relative);

        let content_dir = get_vault_config(db, project).content_dir(db);
        let schema_dir = get_vault_config(db, project).schema_dir(db);

        // Validate: new path must stay within the content/schema directory (depend on the original path)
        if (old_absolute.starts_with(&content_dir) && !new_absolute.starts_with(&content_dir))
          || (old_absolute.starts_with(&schema_dir) && !new_absolute.starts_with(&schema_dir))
        {
          continue;
        }

        // Add file rename operation (only once)
        if !file_rename_map.contains(&old_absolute) {
          let scheme = analysis
            .scheme_map
            .get(&old_absolute)
            .map(|s| s.as_str())
            .unwrap_or("file");
          let old_uri = path_to_uri(&old_absolute, scheme);
          let new_uri = path_to_uri(&new_absolute, scheme);
          changes.push(DocumentChangeOperation::Op(ResourceOp::Rename(
            RenameFile {
              old_uri,
              new_uri,
              options: None,
              annotation_id: None,
            },
          )));
          file_rename_map.insert(old_absolute);
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

  // Text edits before file renames (edits reference old URIs)
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

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use std::path::PathBuf;
  use std::sync::{Arc, Condvar, Mutex};

  use lsp_types::{
    DocumentChangeOperation, DocumentChanges, Position, RenameParams, ResourceOp,
    TextDocumentIdentifier, TextDocumentPositionParams, TextEdit,
  };
  use ropey::Rope;
  use tdr_lang::db::types::{File, FileHandle, Project};
  use tdr_lang::db::{QueryStorage, TypedownDatabase};

  use super::rename;
  use crate::analysis::Analysis;
  use crate::utils::uri::path_to_uri;

  const VAULT_CONFIG: &str = r#"
version: "1"
vault:
  content_dir: content
  schema_dir: schemas
"#;
  const SCHEMA_PERSON: &str = r#"---
_type: schema
properties:
  name:
    type: string
  age:
    type: number
---
"#;
  const CONTENT_ALICE: &str = r#"---
_type: Person
name: Alice
age: 30
---
"#;

  fn cursor(content: &str) -> (String, usize) {
    let offset = content
      .find('|')
      .expect("content must have a cursor marker");
    (content.replacen('|', "", 1), offset)
  }

  fn make_params(
    uri: lsp_types::Uri,
    content: &str,
    offset: usize,
    new_name: &str,
  ) -> RenameParams {
    let rope = Rope::from(content);
    let line = rope.char_to_line(offset);
    let character = offset - rope.line_to_char(line);
    RenameParams {
      text_document_position: TextDocumentPositionParams {
        text_document: TextDocumentIdentifier { uri },
        position: Position {
          line: line as u32,
          character: character as u32,
        },
      },
      new_name: new_name.to_string(),
      work_done_progress_params: Default::default(),
    }
  }

  fn setup(content: &str) -> (Analysis, lsp_types::Uri) {
    let root = PathBuf::from(if cfg!(windows) { "C:\\vault" } else { "/vault" });
    let content_root = root.join("content");
    let schema_root = root.join("schemas");
    let test_path = content_root.join("file.tdr");
    let uri = path_to_uri(&test_path, "file");

    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };

    let config_file = File::new(
      &db,
      FileHandle::Content(root.join("typedown.yaml"), VAULT_CONFIG.to_string()),
    );
    let person_file = File::new(
      &db,
      FileHandle::Content(schema_root.join("Person.tdr"), SCHEMA_PERSON.to_string()),
    );
    let alice_file = File::new(
      &db,
      FileHandle::Content(content_root.join("alice.tdr"), CONTENT_ALICE.to_string()),
    );
    let editing_file = File::new(
      &db,
      FileHandle::Content(test_path.clone(), content.to_string()),
    );

    let files = HashMap::from([
      (root.join("typedown.yaml"), config_file),
      (root.join("schemas/Person.tdr"), person_file),
      (root.join("content/alice.tdr"), alice_file),
      (test_path, editing_file),
    ]);

    let project = Project::new(&db, root, files);
    let analysis = Analysis::new(
      db,
      project,
      Arc::new(HashMap::new()),
      Arc::new(HashMap::new()),
      Arc::new((Mutex::new(1), Condvar::new())),
    );
    (analysis, uri)
  }

  /// Serialize a WorkspaceEdit to a deterministic snapshot string
  fn snapshot(edit: &lsp_types::WorkspaceEdit) -> String {
    let mut lines = vec![];
    if let Some(DocumentChanges::Operations(ops)) = &edit.document_changes {
      for op in ops {
        match op {
          DocumentChangeOperation::Edit(doc_edit) => {
            let uri = doc_edit.text_document.uri.as_str();
            // Strip scheme + vault prefix for readability
            let short = uri.rfind("/vault/").map_or(uri, |i| &uri[i..]);
            for edit in &doc_edit.edits {
              if let lsp_types::OneOf::Left(text_edit) = edit {
                let r = &text_edit.range;
                lines.push(format!(
                  "EDIT {} [{}:{}-{}:{}] -> {:?}",
                  short,
                  r.start.line,
                  r.start.character,
                  r.end.line,
                  r.end.character,
                  text_edit.new_text
                ));
              }
            }
          }
          DocumentChangeOperation::Op(ResourceOp::Rename(rename_file)) => {
            let old = rename_file.old_uri.as_str();
            let new = rename_file.new_uri.as_str();
            let old_short = old.rfind("/vault/").map_or(old, |i| &old[i..]);
            let new_short = new.rfind("/vault/").map_or(new, |i| &new[i..]);
            lines.push(format!("RENAME {} -> {}", old_short, new_short));
          }
          _ => {}
        }
      }
    }
    lines.sort();
    lines.join("\n")
  }

  // Rename an identifier in _type position
  #[test]
  fn rename_ident_in_type_field() {
    let (raw, offset) = cursor(
      r#"---
_type: |Person
name: Alice
---
"#,
    );
    let (analysis, uri) = setup(&raw);
    let params = make_params(uri, &raw, offset, "Human");
    let edit = rename(&analysis, params).expect("should produce edits");
    let snap = snapshot(&edit);

    // Text edits on both files, no file rename
    assert!(snap.contains("EDIT"), "should have text edits");
    assert!(!snap.contains("RENAME"), "should not have file rename");
    // Both the editing file and alice.tdr reference Person
    assert_eq!(
      snap.matches("EDIT").count(),
      2,
      "should have 2 edits:\n{}",
      snap
    );
    assert!(
      snap.lines().all(|l| l.contains("\"Human\"")),
      "all edits should rename to Human:\n{}",
      snap
    );
  }

  // Rename a fref target: text edits come before file rename
  #[test]
  fn rename_fref_snapshot() {
    let (raw, offset) = cursor(
      r#"---
_type: Person
friend: fref("|content/alice.tdr")
---
"#,
    );
    let (analysis, uri) = setup(&raw);
    let edit =
      rename(&analysis, make_params(uri, &raw, offset, "bob")).expect("should produce edits");
    let snap = snapshot(&edit);

    assert!(snap.contains("EDIT"), "should have text edits:\n{}", snap);
    assert!(
      snap.contains("RENAME"),
      "should have file rename:\n{}",
      snap
    );
    assert!(
      snap.contains("bob"),
      "edits should reference new name:\n{}",
      snap
    );
  }

  // Edits are ordered: text edits before file renames
  #[test]
  fn rename_fref_edits_before_renames() {
    let (raw, offset) = cursor(
      r#"---
_type: Person
friend: fref("|content/alice.tdr")
---
"#,
    );
    let (analysis, uri) = setup(&raw);
    let edit =
      rename(&analysis, make_params(uri, &raw, offset, "bob")).expect("should produce edits");

    if let Some(DocumentChanges::Operations(ops)) = &edit.document_changes {
      let mut seen_rename = false;
      for op in ops {
        match op {
          DocumentChangeOperation::Edit(_) => {
            assert!(!seen_rename, "text edit found after file rename");
          }
          DocumentChangeOperation::Op(ResourceOp::Rename(_)) => {
            seen_rename = true;
          }
          _ => {}
        }
      }
      assert!(seen_rename, "should have a file rename");
    }
  }

  // Cursor on a non-renameable position returns None
  #[test]
  fn rename_on_value_returns_none() {
    let (raw, offset) = cursor(
      r#"---
_type: Person
name: |Alice
---
"#,
    );
    let (analysis, uri) = setup(&raw);
    assert!(
      rename(&analysis, make_params(uri, &raw, offset, "Bob")).is_none(),
      "renaming a string value should return None"
    );
  }
}
