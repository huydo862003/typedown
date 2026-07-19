use lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind};
use tdr_lang::db::types::TdrTypeLike;

use tdr_lang::db::TypedownDatabase;
use tdr_lang::db::derived::hir::lower_node;
use tdr_lang::db::derived::parse_file::parse_file;
use tdr_lang::db::derived::typechecker::actual_node_type_member::actual_node_type_member;
use tdr_lang::db::derived::typechecker::expected_node_type_member::expected_node_type_member;
use tdr_lang::db::types::{LiteralValue, MemberType, TypeMember, TypeMemberDescriptors};
use tdr_lang::db::utils::typecheck::lift_type_member_result;
use tdr_lang::syntax::ast::{AstNode, Expr};
use tdr_lang::syntax::syntax_kind::SyntaxKind;

use crate::analysis::Analysis;
use crate::utils::ast::{
  find_ancestor, is_in_value_position, nearest_expr_ancestor, node_at_offset,
};
use crate::utils::position::lsp_position_to_text_offset;
use crate::utils::uri::uri_to_path;

pub fn hover(analysis: &Analysis, params: HoverParams) -> Option<Hover> {
  let db = &analysis.db;
  let project = analysis.project;

  let uri = &params.text_document_position_params.text_document.uri;
  let path = uri_to_path(uri)?;
  let rope = analysis.file_rope(&path)?;
  let offset = lsp_position_to_text_offset(&rope, params.text_document_position_params.position)?;

  let file = *project.files(db).get(&path)?;
  let root = parse_file(db, project, file).ast(db);
  let lookup = offset.saturating_sub(1);
  let node = node_at_offset(root, lookup)?;

  let text = if is_in_value_position(&node) {
    // Value position: show the resolved type of the expression.
    let expr_node = nearest_expr_ancestor(&node)?;
    let hir = lower_node(db, project, file, expr_node);
    let typ = {
      let r = actual_node_type_member(db, hir);
      lift_type_member_result(db, &r)?
    };
    typ.display_name(db)
  } else if find_ancestor(&node, SyntaxKind::YamlMappingEntryKey).is_some() {
    // Key position: show the field name with its declared type.
    let entry_key = find_ancestor(&node, SyntaxKind::YamlMappingEntryKey)?;
    let entry = entry_key.parent()?;
    let entry_value = entry
      .children()
      .find(|c| c.kind() == SyntaxKind::YamlMappingEntryValue)?;
    let value_expr = entry_value.children().find_map(Expr::cast)?;
    let hir = lower_node(db, project, file, value_expr.syntax().clone());
    let member = expected_node_type_member(db, hir).member(db)?;
    let key_text = entry_key.text().trim().to_string();
    format!("{key_text}: {}", member_type_label(db, &member))
  } else {
    return None;
  };

  Some(Hover {
    contents: HoverContents::Markup(MarkupContent {
      kind: MarkupKind::Markdown,
      value: format!("```\n{text}\n```"),
    }),
    range: None,
  })
}

fn member_type_label(db: &TypedownDatabase, member: &TypeMember) -> String {
  let type_str = match member.typ(db) {
    MemberType::Simple(typ) => typ.display_name(db),
    MemberType::Sum(arms) => arms
      .iter()
      .map(|arm| match arm.typ(db) {
        MemberType::Simple(typ) => typ.display_name(db),
        MemberType::Literal(lit) => literal_label(&lit),
        _ => "?".to_string(),
      })
      .collect::<Vec<_>>()
      .join(" | "),
    MemberType::ListOfSum(arms) => {
      let inner = arms
        .iter()
        .map(|arm| match arm.typ(db) {
          MemberType::Simple(typ) => typ.display_name(db),
          MemberType::Literal(lit) => literal_label(&lit),
          _ => "?".to_string(),
        })
        .collect::<Vec<_>>()
        .join(" | ");
      format!("list[{}]", inner)
    }
    MemberType::DictOfSum(arms) => {
      let inner = arms
        .iter()
        .map(|arm| match arm.typ(db) {
          MemberType::Simple(typ) => typ.display_name(db),
          MemberType::Literal(lit) => literal_label(&lit),
          _ => "?".to_string(),
        })
        .collect::<Vec<_>>()
        .join(" | ");
      format!("dict[{}]", inner)
    }
    MemberType::Literal(lit) => literal_label(&lit),
    MemberType::Never => "never".to_string(),
  };
  if member
    .descriptors(db)
    .contains(TypeMemberDescriptors::OPTIONAL)
  {
    format!("{type_str}?")
  } else {
    type_str
  }
}

fn literal_label(lit: &LiteralValue) -> String {
  match lit {
    LiteralValue::Str(s) => format!("\"{s}\""),
    LiteralValue::Bool(b) => b.to_string(),
    LiteralValue::Num(n) => n.clone(),
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use std::path::PathBuf;
  use std::sync::{Arc, Condvar, Mutex};

  use lsp_types::{
    HoverContents, HoverParams, Position, TextDocumentIdentifier, TextDocumentPositionParams, Uri,
    WorkDoneProgressParams,
  };
  use ropey::Rope;
  use tdr_lang::db::types::{File, FileHandle, Project};
  use tdr_lang::db::{QueryStorage, TypedownDatabase};

  use crate::analysis::Analysis;
  use crate::utils::uri::path_to_uri;

  use super::hover;

  const VAULT_CONFIG: &str = r#"version: "1"
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
  nickname:
    type: string
    optional: true
---
"#;

  // Accept a text with `|` marker
  // Return the original text with the offset of the marker
  fn cursor(content: &str) -> (String, usize) {
    let offset = content
      .find('|')
      .expect("content must have a cursor marker");
    (content.replacen('|', "", 1), offset)
  }

  // Prepare the LSP client hover request params
  fn make_params(uri: Uri, content: &str, offset: usize) -> HoverParams {
    let rope = Rope::from(content);
    let line = rope.char_to_line(offset);
    let character = offset - rope.line_to_char(line);
    HoverParams {
      text_document_position_params: TextDocumentPositionParams {
        text_document: TextDocumentIdentifier { uri },
        position: Position {
          line: line as u32,
          character: character as u32,
        },
      },
      work_done_progress_params: WorkDoneProgressParams::default(),
    }
  }

  // Project to test against
  // Accept a `content` as the current editing content
  fn setup(content: &str) -> (Analysis, Uri) {
    let root = PathBuf::from(if cfg!(windows) { "C:\\vault" } else { "/vault" });
    let content_path = root.join("content/file.tdr");
    let uri = path_to_uri(&content_path, "file");

    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };

    let config_file = File::new(&db, FileHandle::Content(VAULT_CONFIG.to_string()));
    let person_file = File::new(&db, FileHandle::Content(SCHEMA_PERSON.to_string()));
    let editing_file = File::new(&db, FileHandle::Content(content.to_string()));

    let files = HashMap::from([
      (root.join("typedown.yaml"), config_file),
      (root.join("schemas/Person.tdr"), person_file),
      (content_path, editing_file),
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

  fn hover_text(analysis: &Analysis, uri: Uri, content: &str, offset: usize) -> Option<String> {
    let params = make_params(uri, content, offset);
    let result = hover(analysis, params)?;
    if let HoverContents::Markup(markup) = result.contents {
      Some(markup.value)
    } else {
      None
    }
  }

  #[test]
  fn hover_on_value_shows_resolved_type() {
    let (content, offset) = cursor(
      r#"---
_type: Person
name: Ali|ce
---
"#,
    );
    let (analysis, uri) = setup(&content);
    let text = hover_text(&analysis, uri, &content, offset).expect("expected hover");
    assert!(text.contains("string"), "expected string type, got: {text}");
  }

  #[test]
  fn hover_on_key_shows_field_type() {
    let (content, offset) = cursor(
      r#"---
_type: Person
na|me: Alice
---
"#,
    );
    let (analysis, uri) = setup(&content);
    let text = hover_text(&analysis, uri, &content, offset).expect("expected hover");
    assert!(text.contains("name"), "expected field name, got: {text}");
    assert!(text.contains("string"), "expected field type, got: {text}");
  }

  #[test]
  fn hover_on_optional_key_shows_optional_marker() {
    let (content, offset) = cursor(
      r#"---
_type: Person
nick|name: Bob
---
"#,
    );
    let (analysis, uri) = setup(&content);
    let text = hover_text(&analysis, uri, &content, offset).expect("expected hover");
    assert!(
      text.contains("nickname"),
      "expected field name, got: {text}"
    );
    assert!(
      text.contains("string?"),
      "expected optional marker, got: {text}"
    );
  }
}
