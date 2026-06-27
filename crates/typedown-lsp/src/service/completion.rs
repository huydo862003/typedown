use std::any::Any;

use lsp_types::{CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse};
use typedown_db::TypedownDatabase;
use typedown_db::derived::name_resolver::members::members;
use typedown_db::derived::parse_file::parse_file;
use typedown_db::derived::typechecker::get_symbol_type::get_symbol_type;
use typedown_db::types::{Project, Scope, SymbolKind, TdrProductType};
use typedown_types::syntax_kind::SyntaxKind;
use typedown_syntax::red::RedNode;

use crate::analysis::Analysis;
use crate::utils::ast::node_at_offset;
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
  let node = node_at_offset(root, offset)?;

  // Cursor in a _type value: suggest schema names.
  if is_type_value_position(&node) {
    return Some(CompletionResponse::Array(schema_completions(db, project)));
  }

  // Cursor in a mapping key: suggest field names from the declared schema.
  if let Some(schema_name) = enclosing_mapping_type(&node) {
    return Some(CompletionResponse::Array(field_completions(
      db,
      project,
      &schema_name,
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

/// If the cursor is inside a `YamlMappingEntryKey`, return the `_type` value
/// of the enclosing mapping (the schema name declared there).
fn enclosing_mapping_type(node: &RedNode) -> Option<String> {
  find_ancestor(node, SyntaxKind::YamlMappingEntryKey)?;
  let mapping = find_ancestor(node, SyntaxKind::YamlMapping)?;

  for entry in mapping.children() {
    if entry.kind() != SyntaxKind::YamlMappingEntry {
      continue;
    }
    let mut children = entry.children();
    let key = children.find(|child| child.kind() == SyntaxKind::YamlMappingEntryKey)?;
    if key.text().trim() != "_type" {
      continue;
    }
    let value = children.find(|child| child.kind() == SyntaxKind::YamlMappingEntryValue)?;
    return Some(value.text().trim().to_string());
  }

  None
}

/// Walk up the tree to find the nearest ancestor with the given kind.
fn find_ancestor(node: &RedNode, kind: SyntaxKind) -> Option<RedNode> {
  let mut current = node.parent()?;
  loop {
    if current.kind() == kind {
      return Some(current);
    }
    current = current.parent()?;
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

/// Suggest field names declared by the named schema.
fn field_completions(
  db: &TypedownDatabase,
  project: Project,
  schema_name: &str,
) -> Vec<CompletionItem> {
  let scope = Scope::project_scope(db, project);
  let project_members = members(db, scope);
  let symbol = match project_members.members(db).get(schema_name) {
    Some(sym) => *sym,
    None => return vec![],
  };

  let type_result = get_symbol_type(db, symbol);
  let typ = match type_result.typ(db) {
    Some(typ) => typ,
    None => return vec![],
  };

  let product = match (typ.as_ref() as &dyn Any).downcast_ref::<TdrProductType>() {
    Some(product) => product,
    None => return vec![],
  };

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
