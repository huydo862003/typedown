use tdr_lang::db::types::TdrTypeLike;

use lsp_types::{CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse};
use tdr_lang::db::TypedownDatabase;
use tdr_lang::db::derived::evaluate::evaluate_type::evaluate_type;
use tdr_lang::db::derived::hir::lower_node;
use tdr_lang::db::derived::name_resolver::file_symbol::file_symbol;
use tdr_lang::db::derived::name_resolver::members::members;
use tdr_lang::db::derived::parse_file::parse_file;
use tdr_lang::db::derived::typechecker::expected_node_type::expected_node_type;
use tdr_lang::db::derived::typechecker::get_symbol_type::get_symbol_type;
use tdr_lang::db::types::{
  File, MemberType, Project, Scope, SymbolKind, TdrProductType, TypeMember, TypeMemberDescriptors,
};
use tdr_lang::db::utils::schema_name_in_mapping;
use tdr_lang::db::utils::typecheck::lift_type_member_result;
use tdr_lang::syntax::ast::{AstNode, Expr};
use tdr_lang::syntax::red::RedNode;
use tdr_lang::syntax::syntax_kind::SyntaxKind;

use crate::analysis::Analysis;
use crate::utils::ast::{find_ancestor, is_in_value_position, node_at_offset};
use crate::utils::position::lsp_position_to_text_offset;
use crate::utils::uri::uri_to_path;

pub fn completion(analysis: &Analysis, params: CompletionParams) -> Option<CompletionResponse> {
  let db = &analysis.db;
  let project = analysis.project;

  let path = uri_to_path(&params.text_document_position.text_document.uri)?;
  let rope = analysis.file_rope(&path)?;
  let offset = lsp_position_to_text_offset(&rope, params.text_document_position.position)?;

  let file = *project.files(db).get(&path)?;
  let root = parse_file(db, project, file).ast(db);
  // Use offset-1 so the cursor position (between characters) resolves to the token just typed.
  let lookup = offset.saturating_sub(1);
  let node = node_at_offset(root, lookup)?;

  // Cursor in a _type value: suggest schema names.
  if is_type_value_position(&node) {
    return Some(CompletionResponse::Array(schema_completions(db, project)));
  }

  // Cursor inside a fref() string argument. Suggest .tdr files matching the field's declared type.
  if is_fref_arg_position(&node) {
    return Some(CompletionResponse::Array(fref_completions(
      db, project, file, &node,
    )));
  }

  // Cursor in a field value: suggest value completions (booleans, null for optional fields).
  if let Some(items) = value_completions(db, project, file, &node) {
    return Some(CompletionResponse::Array(items));
  }

  // Cursor in a mapping key: suggest field names from the declared schema.
  if let Some(product) = enclosing_mapping_product(db, project, file, &node) {
    return Some(CompletionResponse::Array(field_completions_from_type(
      db, &product,
    )));
  }

  None
}

/// Returns true if `node` is inside the value position of a `_type` mapping entry.
fn is_type_value_position(node: &RedNode) -> bool {
  let Some(entry) = find_ancestor(node, SyntaxKind::YamlMappingEntry) else {
    return false;
  };
  let Some(key) = entry
    .children()
    .find(|child| child.kind() == SyntaxKind::YamlMappingEntryKey)
  else {
    return false;
  };
  key.text().trim() == "_type"
}

/// Returns true if `node` is inside the string argument of a `fref()` call.
fn is_fref_arg_position(node: &RedNode) -> bool {
  // Walk up to find an enclosing StrLit, then a CallExpr above it.
  let str_lit = find_ancestor(node, SyntaxKind::StrLit);
  let call = match str_lit {
    Some(ref lit) => find_ancestor(lit, SyntaxKind::CallExpr),
    None => find_ancestor(node, SyntaxKind::CallExpr),
  };
  let Some(call) = call else {
    return false;
  };
  // Check callee text is "fref".
  call
    .children()
    .next()
    .is_some_and(|callee| callee.text().trim() == "fref")
}

/// Suggest .tdr file paths whose type is compatible with the declared field type.
/// Falls back to all .tdr files if no declared type can be resolved.
fn fref_completions(
  db: &TypedownDatabase,
  project: Project,
  file: File,
  node: &RedNode,
) -> Vec<CompletionItem> {
  // Resolve the expected type for the field containing this fref() call.
  let expected_type =
    declared_field(db, project, file, node).and_then(|member| match member.typ(db) {
      MemberType::Simple(typ) => Some(typ),
      _ => None,
    });

  let root = project.root_dir(db);
  project
    .files(db)
    .iter()
    .filter(|(path, _)| path.extension().is_some_and(|ext| ext == "tdr"))
    .filter(|(_, target_file)| {
      // If we have an expected type, only include files whose type is compatible.
      let Some(ref expected) = expected_type else {
        return true;
      };
      let sym = match file_symbol(db, project, **target_file).value(db) {
        Some(sym) => sym,
        None => return false,
      };
      let file_type = match lift_type_member_result(db, &get_symbol_type(db, sym)) {
        Some(typ) => typ,
        None => return false,
      };
      expected.is_compatible_with(db, &file_type)
    })
    .filter_map(|(path, _)| path.strip_prefix(&root).ok().map(|rel| rel.to_path_buf()))
    .map(|rel| CompletionItem {
      label: rel.to_string_lossy().into_owned(),
      kind: Some(CompletionItemKind::FILE),
      ..Default::default()
    })
    .collect()
}

/// Resolve the product type of the mapping the cursor key belongs to.
/// Returns None if the cursor is not on a key or no type can be resolved.
fn enclosing_mapping_product(
  db: &TypedownDatabase,
  project: Project,
  file: File,
  node: &RedNode,
) -> Option<TdrProductType> {
  find_ancestor(node, SyntaxKind::YamlMappingEntryKey)?;
  let mapping = find_ancestor(node, SyntaxKind::YamlMapping)?;

  // Explicit _type in this mapping.
  if let Some(schema_name) = schema_name_in_mapping(&mapping) {
    let scope = Scope::project_scope(db, project);
    let symbol = *members(db, scope).members(db).get(&schema_name)?;
    let typ = evaluate_type(db, symbol).typ(db)?;
    return typ.as_tdr_product_type().cloned();
  }

  // No explicit _type. Try resolving via the parent field's declared type.
  let mapping_expr = Expr::cast(mapping.clone())?;
  let hir = lower_node(db, project, file, mapping_expr.syntax().clone());
  let member = expected_node_type(db, hir).member(db)?;
  let typ = match member.typ(db) {
    MemberType::Simple(typ) => typ,
    _ => return None,
  };
  typ.as_tdr_product_type().cloned()
}

/// If the cursor is in a field value, return value completions.
/// Always suggests `true`/`false` (valid in any expression),
/// suggests `null` for optional fields.
fn value_completions(
  db: &TypedownDatabase,
  project: Project,
  file: File,
  node: &RedNode,
) -> Option<Vec<CompletionItem>> {
  // Must be directly inside a value position (not a key that happens to be nested in a value).
  if !is_in_value_position(node) {
    return None;
  }

  // true and false are keywords usable in any value/expression position.
  let mut items = vec![keyword_item("true"), keyword_item("false")];

  // Suggest null only for optional fields.
  if let Some(field) = declared_field(db, project, file, node)
    && field
      .descriptors(db)
      .contains(TypeMemberDescriptors::OPTIONAL)
  {
    items.push(keyword_item("null"));
  }

  Some(items)
}

/// Resolve the `TypeMember` for the field whose value the cursor is currently in.
fn declared_field(
  db: &TypedownDatabase,
  project: Project,
  file: File,
  node: &RedNode,
) -> Option<TypeMember> {
  // Find the value expression node inside the enclosing YamlMappingEntryValue.
  let entry_value = find_ancestor(node, SyntaxKind::YamlMappingEntryValue)?;
  let value_expr = entry_value.children().find_map(Expr::cast)?;
  let hir = lower_node(db, project, file, value_expr.syntax().clone());
  expected_node_type(db, hir).member(db)
}

/// Build a keyword completion item (true, false, null).
fn keyword_item(label: &str) -> CompletionItem {
  CompletionItem {
    label: label.to_string(),
    kind: Some(CompletionItemKind::KEYWORD),
    ..Default::default()
  }
}

/// Suggest all user-defined schema names visible in the project scope.
fn schema_completions(db: &TypedownDatabase, project: Project) -> Vec<CompletionItem> {
  let scope = Scope::project_scope(db, project);
  members(db, scope)
    .members(db)
    .iter()
    .filter(|(_, sym)| matches!(sym.kind(db), SymbolKind::UserDefinedSchema(..)))
    .map(|(name, _)| CompletionItem {
      label: name.clone(),
      kind: Some(CompletionItemKind::CLASS),
      ..Default::default()
    })
    .collect()
}

/// Suggest field names from a resolved product type.
fn field_completions_from_type(
  db: &TypedownDatabase,
  product: &TdrProductType,
) -> Vec<CompletionItem> {
  product
    .fields(db)
    .keys()
    .map(|field| CompletionItem {
      label: field.clone(),
      kind: Some(CompletionItemKind::FIELD),
      ..Default::default()
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use std::path::PathBuf;
  use std::sync::{Arc, Condvar, Mutex};

  use lsp_types::{
    CompletionParams, CompletionResponse, PartialResultParams, Position, TextDocumentIdentifier,
    TextDocumentPositionParams, Uri, WorkDoneProgressParams,
  };
  use ropey::Rope;
  use tdr_lang::db::types::{File, FileHandle, Project};
  use tdr_lang::db::{QueryStorage, TypedownDatabase};

  use crate::analysis::Analysis;
  use crate::utils::uri::path_to_uri;

  use super::completion;

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
  verified:
    type: boolean
  nickname:
    type: string
    optional: true
---
"#;
  const SCHEMA_EVENT: &str = r#"---
_type: schema
properties:
  name:
    type: string
  date:
    type: date
---
"#;

  // Schema with a nested inline object field (no named type reference).
  const SCHEMA_PERSON_WITH_ADDRESS: &str = r#"---
_type: schema
properties:
  name:
    type: string
  address:
    type:
      street:
        type: string
      city:
        type: string
---
"#;

  /// Strip the `|` cursor marker from content and return its char offset.
  fn cursor(content: &str) -> (String, usize) {
    let offset = content
      .find('|')
      .expect("content must have a '|' cursor marker");
    (content.replacen('|', "", 1), offset)
  }

  /// Build CompletionParams from a URI, content string, and char offset.
  fn make_params(uri: Uri, content: &str, offset: usize) -> CompletionParams {
    let rope = Rope::from(content);
    let line = rope.char_to_line(offset);
    let character = offset - rope.line_to_char(line);
    CompletionParams {
      text_document_position: TextDocumentPositionParams {
        text_document: TextDocumentIdentifier { uri },
        position: Position {
          line: line as u32,
          character: character as u32,
        },
      },
      work_done_progress_params: WorkDoneProgressParams::default(),
      partial_result_params: PartialResultParams::default(),
      context: None,
    }
  }

  /// Build an in-memory vault with Person and Event schemas, plus the given content file.
  fn setup(content: &str) -> (Analysis, Uri) {
    let root = PathBuf::from(if cfg!(windows) { "C:\\vault" } else { "/vault" });
    let content_path = root.join("content/file.tdr");
    let uri = path_to_uri(&content_path, "file");

    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };

    let config_file = File::new(&db, FileHandle::Content(VAULT_CONFIG.to_string()));
    let person_file = File::new(&db, FileHandle::Content(SCHEMA_PERSON.to_string()));
    let event_file = File::new(&db, FileHandle::Content(SCHEMA_EVENT.to_string()));
    let person_with_address_file = File::new(
      &db,
      FileHandle::Content(SCHEMA_PERSON_WITH_ADDRESS.to_string()),
    );
    let content_file = File::new(&db, FileHandle::Content(content.to_string()));

    let files = HashMap::from([
      (root.join("typedown.yaml"), config_file),
      (root.join("schemas/Person.tdr"), person_file),
      (root.join("schemas/Event.tdr"), event_file),
      (
        root.join("schemas/PersonWithAddress.tdr"),
        person_with_address_file,
      ),
      (content_path, content_file),
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

  #[test]
  fn schema_name_completion_in_type_value() {
    let (content, offset) = cursor(
      r#"---
_type: |
---
"#,
    );
    let (analysis, uri) = setup(&content);
    let params = make_params(uri, &content, offset);

    let response = completion(&analysis, params);
    let Some(CompletionResponse::Array(items)) = response else {
      panic!("expected completion items");
    };
    let labels: Vec<&str> = items.iter().map(|item| item.label.as_str()).collect();
    assert!(labels.contains(&"Person"), "should suggest Person schema");
    assert!(labels.contains(&"Event"), "should suggest Event schema");
  }

  #[test]
  fn schema_name_completion_while_partially_typed() {
    // Cursor in the middle of a partially typed schema name.
    let (content, offset) = cursor(
      r#"---
_type: Per|
---
"#,
    );
    let (analysis, uri) = setup(&content);
    let params = make_params(uri, &content, offset);

    let response = completion(&analysis, params);
    let Some(CompletionResponse::Array(items)) = response else {
      panic!("expected completion items");
    };
    let labels: Vec<&str> = items.iter().map(|item| item.label.as_str()).collect();
    assert!(labels.contains(&"Person"), "should still suggest Person");
  }

  #[test]
  fn field_completion_based_on_declared_type() {
    // Cursor after typing a partial key, _type already set.
    let (content, offset) = cursor(
      r#"---
_type: Person
na|:
---
"#,
    );
    let (analysis, uri) = setup(&content);
    let params = make_params(uri, &content, offset);

    let response = completion(&analysis, params);
    let Some(CompletionResponse::Array(items)) = response else {
      panic!("expected field completions");
    };
    let labels: Vec<&str> = items.iter().map(|item| item.label.as_str()).collect();
    assert!(labels.contains(&"name"), "should suggest 'name' field");
    assert!(labels.contains(&"age"), "should suggest 'age' field");
  }

  #[test]
  fn field_completion_when_type_declared_after_other_fields() {
    // _type appears after the cursor position in the mapping.
    let (content, offset) = cursor(
      r#"---
name: Alice
ag|:
_type: Person
---
"#,
    );
    let (analysis, uri) = setup(&content);
    let params = make_params(uri, &content, offset);

    let response = completion(&analysis, params);
    let Some(CompletionResponse::Array(items)) = response else {
      panic!("expected field completions");
    };
    let labels: Vec<&str> = items.iter().map(|item| item.label.as_str()).collect();
    assert!(labels.contains(&"name"), "should suggest 'name' field");
    assert!(labels.contains(&"age"), "should suggest 'age' field");
  }

  #[test]
  fn no_field_completion_without_type() {
    // No _type in mapping: no field completions expected.
    let (content, offset) = cursor(
      r#"---
na|:
---
"#,
    );
    let (analysis, uri) = setup(&content);
    let params = make_params(uri, &content, offset);

    let response = completion(&analysis, params);
    let is_empty = matches!(response, None)
      || matches!(response, Some(CompletionResponse::Array(ref items)) if items.is_empty());
    assert!(is_empty, "should not suggest fields when _type is absent");
  }

  #[test]
  fn no_completion_in_markdown_body() {
    // Cursor in the markdown body below the frontmatter: no completions.
    let (content, offset) = cursor(
      r#"---
_type: Person
name: Alice
---

Some bod|y text.
"#,
    );
    let (analysis, uri) = setup(&content);
    let params = make_params(uri, &content, offset);

    let response = completion(&analysis, params);
    let is_empty = matches!(response, None)
      || matches!(response, Some(CompletionResponse::Array(ref items)) if items.is_empty());
    assert!(is_empty, "should not suggest anything in the markdown body");
  }

  #[test]
  fn boolean_keywords_suggested_in_any_value_position() {
    // true/false are keywords usable in any value position, not limited to boolean-typed fields.
    let (content, offset) = cursor(
      r#"---
_type: Person
name: tru|
---
"#,
    );
    let (analysis, uri) = setup(&content);
    let params = make_params(uri, &content, offset);

    let response = completion(&analysis, params);
    let Some(CompletionResponse::Array(items)) = response else {
      panic!("expected value completions");
    };
    let labels: Vec<&str> = items.iter().map(|item| item.label.as_str()).collect();
    assert!(
      labels.contains(&"true"),
      "should suggest 'true' in any value position"
    );
    assert!(
      labels.contains(&"false"),
      "should suggest 'false' in any value position"
    );
    assert!(
      !labels.contains(&"null"),
      "non-optional field should not suggest 'null'"
    );
  }

  #[test]
  fn null_completion_for_optional_field() {
    // Cursor in the value of an optional field: suggest null.
    let (content, offset) = cursor(
      r#"---
_type: Person
nickname: nu|
---
"#,
    );
    let (analysis, uri) = setup(&content);
    let params = make_params(uri, &content, offset);

    let response = completion(&analysis, params);
    let Some(CompletionResponse::Array(items)) = response else {
      panic!("expected value completions");
    };
    let labels: Vec<&str> = items.iter().map(|item| item.label.as_str()).collect();
    assert!(
      labels.contains(&"null"),
      "optional field should suggest 'null'"
    );
  }

  // A schema with a field typed as another schema (Person).
  const SCHEMA_DIRECTORY: &str = r#"---
_type: schema
properties:
  featured:
    type: Person
---
"#;

  const CONTENT_ALICE: &str = r#"---
_type: Person
name: Alice
age: 30
---
"#;

  const CONTENT_BIRTHDAY: &str = r#"---
_type: Event
name: Birthday
date: 2024-01-01
---
"#;

  /// Build a vault with Person, Event, Directory schemas plus two content files.
  fn setup_with_content(content: &str) -> (Analysis, Uri) {
    let root = PathBuf::from(if cfg!(windows) { "C:\\vault" } else { "/vault" });
    let content_path = root.join("content/file.tdr");
    let uri = path_to_uri(&content_path, "file");

    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };

    let config_file = File::new(&db, FileHandle::Content(VAULT_CONFIG.to_string()));
    let person_file = File::new(&db, FileHandle::Content(SCHEMA_PERSON.to_string()));
    let event_file = File::new(&db, FileHandle::Content(SCHEMA_EVENT.to_string()));
    let directory_file = File::new(&db, FileHandle::Content(SCHEMA_DIRECTORY.to_string()));
    let alice_file = File::new(&db, FileHandle::Content(CONTENT_ALICE.to_string()));
    let birthday_file = File::new(&db, FileHandle::Content(CONTENT_BIRTHDAY.to_string()));
    let editing_file = File::new(&db, FileHandle::Content(content.to_string()));

    let files = HashMap::from([
      (root.join("typedown.yaml"), config_file),
      (root.join("schemas/Person.tdr"), person_file),
      (root.join("schemas/Event.tdr"), event_file),
      (root.join("schemas/Directory.tdr"), directory_file),
      (root.join("content/alice.tdr"), alice_file),
      (root.join("content/birthday.tdr"), birthday_file),
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

  #[test]
  fn fref_completion_filters_by_declared_field_type() {
    // The 'featured' field on Directory expects type Person.
    // Only content/alice.tdr (_type: Person) should be suggested, not content/birthday.tdr (_type: Event).
    let (content, offset) = cursor(
      r#"---
_type: Directory
featured: fref("|")
---
"#,
    );
    let (analysis, uri) = setup_with_content(&content);
    let params = make_params(uri, &content, offset);

    let response = completion(&analysis, params);
    let Some(CompletionResponse::Array(items)) = response else {
      panic!("expected fref completions");
    };
    let labels: Vec<String> = items.iter().map(|item| item.label.clone()).collect();
    assert!(
      labels.iter().any(|label| label.contains("alice")),
      "should suggest alice.tdr (Person type), got: {:?}",
      labels
    );
    assert!(
      !labels.iter().any(|label| label.contains("birthday")),
      "should not suggest birthday.tdr (Event type), got: {:?}",
      labels
    );
  }

  // Cursor on a key inside a nested mapping whose type is inferred from the parent schema field.
  #[test]
  fn field_completion_in_nested_mapping_without_type() {
    let (content, offset) = cursor(
      r#"---
_type: PersonWithAddress
name: Alice
address:
  str|:
---
"#,
    );
    let (analysis, uri) = setup(&content);
    let params = make_params(uri, &content, offset);

    let response = completion(&analysis, params);
    let Some(CompletionResponse::Array(items)) = response else {
      panic!("expected field completions for nested address mapping");
    };
    let labels: Vec<&str> = items.iter().map(|item| item.label.as_str()).collect();
    assert!(
      labels.contains(&"street"),
      "should suggest 'street' from nested address type, got: {:?}",
      labels
    );
    assert!(
      labels.contains(&"city"),
      "should suggest 'city' from nested address type, got: {:?}",
      labels
    );
  }
}
