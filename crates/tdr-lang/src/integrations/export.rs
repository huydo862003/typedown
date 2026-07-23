//! Export typedown resources

use std::collections::HashMap;

use tdr_types::either::Either;

use crate::db::TypedownDatabase;
use crate::db::derived::evaluate::evaluate_node::evaluate_node;
use crate::db::derived::evaluate::evaluate_resource::evaluate_resource;
use crate::db::derived::get_vault_config::get_vault_config;
use crate::db::derived::hir::lower_node;
use crate::db::derived::name_resolver::file_symbol::file_symbol;
use crate::db::derived::name_resolver::referee::referee;
use crate::db::derived::parse_file::parse_file;
use crate::db::types::{File, HirValue, Project, Symbol, SymbolKind, TdrObjectEnum, TdrObjectLike};
use crate::syntax::ast::{AstNode, MdBody, MdToggleList, SourceFile};
use crate::syntax::red::RedNode;
use crate::syntax::syntax_kind::SyntaxKind;

/// Structured export result
pub struct ExportedResource {
  /// Frontmatter fields as key-value pairs
  pub header: HashMap<String, ExportedValue>,
  /// Commonmark-compatible markdown body
  pub content: String,
}

/// An exported field value
pub enum ExportedValue {
  String(String),
  Number(f64),
  Bool(bool),
  List(Vec<ExportedValue>),
  Object(HashMap<String, ExportedValue>),
  Null,
}

/// Export a resource file as structured header and commonmark content
pub fn export_resource(
  db: &TypedownDatabase,
  project: Project,
  file: File,
) -> Option<ExportedResource> {
  let (symbol, obj) = resolve_resource(db, project, file)?;
  let mut header = export_header(db, &obj);

  // FIXME: _type is not in product fields, add from symbol
  header.insert(
    "_type".to_string(),
    ExportedValue::String(symbol.name(db).to_string()),
  );

  // Walk the AST and translate to somewhat commonmark-conformant markdown
  let parse_result = parse_file(db, project, file);
  let root = parse_result.ast(db);
  let source_file = SourceFile::cast(root)?;
  let body = source_file.body()?;
  let content = export_markdown_body(db, project, file, &body);

  Some(ExportedResource { header, content })
}

/// Extract frontmatter fields, excluding _content
fn export_header(db: &TypedownDatabase, obj: &TdrObjectEnum) -> HashMap<String, ExportedValue> {
  let mut header = HashMap::new();

  let product = match obj {
    TdrObjectEnum::TdrProductObj(product) => product,
    _ => return header,
  };

  for (key, field) in product.fields(db) {
    // Skip _content, available in ExportedResource.content
    if key == "_content" {
      continue;
    }
    if let Some(value) = evaluate_lazy_field(db, field.clone()) {
      header.insert(key.clone(), export_value(db, &value));
    }
  }

  header
}

fn export_value(db: &TypedownDatabase, obj: &TdrObjectEnum) -> ExportedValue {
  match obj {
    TdrObjectEnum::TdrStrObj(str_obj) => ExportedValue::String(str_obj.value(db)),
    TdrObjectEnum::TdrNumObj(num_obj) => ExportedValue::Number(num_obj.value(db)),
    TdrObjectEnum::TdrBoolObj(bool_obj) => ExportedValue::Bool(bool_obj.value(db)),
    TdrObjectEnum::TdrListObj(list_obj) => {
      let items = list_obj
        .items(db)
        .iter()
        .filter_map(|item| evaluate_lazy_field(db, item.clone()))
        .map(|obj| export_value(db, &obj))
        .collect();
      ExportedValue::List(items)
    }
    TdrObjectEnum::TdrProductObj(product) => {
      let mut map = HashMap::new();
      for (key, field) in product.fields(db) {
        if let Some(value) = evaluate_lazy_field(db, field.clone()) {
          map.insert(key.clone(), export_value(db, &value));
        }
      }
      ExportedValue::Object(map)
    }
    _ => ExportedValue::Null,
  }
}

fn export_markdown_body(
  db: &TypedownDatabase,
  project: Project,
  file: File,
  body: &MdBody,
) -> String {
  let mut out = String::new();

  for block in body.block_elements() {
    emit_md_block(db, project, file, block.syntax(), &mut out);
  }

  if !out.ends_with('\n') {
    out.push('\n');
  }
  out
}

fn emit_md_block(
  db: &TypedownDatabase,
  project: Project,
  file: File,
  node: &RedNode,
  out: &mut String,
) {
  match node.kind() {
    SyntaxKind::MdToggleList => emit_md_toggle_list(db, project, file, node, out),
    _ => emit_md_node(db, project, file, node, out),
  }
}

/// Emit a toggle list as HTML <details><summary>...</summary>...</details>
fn emit_md_toggle_list(
  db: &TypedownDatabase,
  project: Project,
  file: File,
  node: &RedNode,
  out: &mut String,
) {
  let Some(list) = MdToggleList::cast(node.clone()) else {
    return;
  };

  for item in list.items() {
    out.push_str("<details>\n");

    if let Some(summary) = item.summary() {
      out.push_str("<summary>");
      emit_md_node(db, project, file, summary.syntax(), out);
      out.push_str("</summary>\n\n");
    }

    if let Some(details) = item.details() {
      for block in details.block_elements() {
        emit_md_block(db, project, file, block.syntax(), out);
      }
    }

    out.push_str("</details>\n\n");
  }
}

/// Emit a node, translating fref interpolations to markdown links
fn emit_md_node(
  db: &TypedownDatabase,
  project: Project,
  file: File,
  node: &RedNode,
  out: &mut String,
) {
  // Leaf token
  if node.as_token().is_some() {
    out.push_str(&node.text());
    return;
  }

  // Interpolation fragment: Try to resolve as fref link
  if node.kind() == SyntaxKind::InterpFragment {
    if let Some(link) = try_resolve_fref(db, project, file, node) {
      out.push_str(&link);
      return;
    }
    // Not a fref, emit source text as fallback
    out.push_str(&node.text());
    return;
  }

  // Composite node: Recurse into children
  for child in node.children() {
    emit_md_node(db, project, file, &child, out);
  }
}

/// Resolve a fref interpolation to a markdown link
fn try_resolve_fref(
  db: &TypedownDatabase,
  project: Project,
  file: File,
  node: &RedNode,
) -> Option<String> {
  let hir = lower_node(db, project, file, node.clone());
  let target_symbol = referee(db, hir).value(db)?;

  let name = resolve_display_name(db, project, &target_symbol);

  // Build URL-friendly path relative to content_dir
  let target_path = match target_symbol.kind(db) {
    SymbolKind::UserDefinedResource(_, target_file)
    | SymbolKind::UserDefinedSchema(_, target_file) => {
      let handle = target_file.handle(db);
      let path = handle.path()?;
      let config = get_vault_config(db, project);
      let content_dir = config.content_dir(db);
      let relative = path.strip_prefix(&content_dir).unwrap_or(path);
      let path_str = relative.to_string_lossy();
      let without_ext = path_str.strip_suffix(".tdr").unwrap_or(&path_str);
      format!("/{without_ext}")
    }
    _ => return None,
  };

  Some(format!("[{name}]({target_path})"))
}

/// Get a display name for a symbol: Try _label, then name field, then file stem
fn resolve_display_name(db: &TypedownDatabase, project: Project, symbol: &Symbol) -> String {
  let kind = symbol.kind(db);

  // Try _label or name from the evaluated resource
  if let SymbolKind::UserDefinedResource(_, target_file) = &kind
    && let Some(target_symbol) = file_symbol(db, project, *target_file).value(db)
    && let Some(obj) = evaluate_resource(db, target_symbol).value(db)
  {
    let label_or_name = obj
      .get_owned_field(db, "_label")
      .or_else(|| obj.get_owned_field(db, "name"));
    if let Some(str_obj) = label_or_name
      .as_ref()
      .and_then(|field| field.as_tdr_str_obj())
    {
      return str_obj.value(db);
    }
  }

  // Fallback: File stem
  match &kind {
    SymbolKind::UserDefinedResource(_, target_file)
    | SymbolKind::UserDefinedSchema(_, target_file) => target_file
      .handle(db)
      .path()
      .and_then(|path| path.file_stem())
      .and_then(|stem| stem.to_str())
      .unwrap_or("unknown")
      .to_string(),
    _ => symbol.name(db).to_string(),
  }
}

fn evaluate_lazy_field(
  db: &TypedownDatabase,
  field: Either<HirValue, TdrObjectEnum>,
) -> Option<TdrObjectEnum> {
  match field {
    Either::Right(obj) => Some(obj),
    Either::Left(hir) => evaluate_node(db, hir).value(db),
  }
}

fn resolve_resource(
  db: &TypedownDatabase,
  project: Project,
  file: File,
) -> Option<(Symbol, TdrObjectEnum)> {
  let symbol = file_symbol(db, project, file).value(db)?;
  if !matches!(symbol.kind(db), SymbolKind::UserDefinedResource(..)) {
    return None;
  }
  let obj = evaluate_resource(db, symbol).value(db)?;
  Some((symbol, obj))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::db::fixtures::load_vault_fixture;

  #[test]
  fn exports_header_fields() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "content/valid_person.tdr");
    let result = export_resource(&db, project, file);
    let exported = result.expect("should export");
    assert!(!exported.header.is_empty(), "header should have fields");
    assert!(
      exported.header.contains_key("_type"),
      "should contain _type"
    );
    assert!(
      !exported.header.contains_key("_content"),
      "should not contain _content"
    );
  }

  #[test]
  fn exports_content_body() {
    let (db, project, file) =
      load_vault_fixture("evaluate/my_vault", "content/md_with_content.tdr");
    let result = export_resource(&db, project, file);
    let exported = result.expect("should export");
    assert!(
      exported.content.contains("Hello world"),
      "content should contain body text: {}",
      exported.content
    );
  }

  #[test]
  fn exports_markdown_body() {
    let (db, project, file) =
      load_vault_fixture("evaluate/my_vault", "content/md_with_content.tdr");
    let exported = export_resource(&db, project, file).expect("should export");
    assert_eq!(exported.content, "Hello world\n");
  }

  #[test]
  fn exports_all_markdown_elements() {
    let (db, project, file) =
      load_vault_fixture("evaluate/my_vault", "content/all_md_elements.tdr");
    let exported = export_resource(&db, project, file).expect("should export");
    let content = &exported.content;
    // Verify key elements are present in the exported content
    assert!(content.contains("# Heading 1"), "should contain h1");
    assert!(content.contains("## Heading 2"), "should contain h2");
    assert!(content.contains("**bold**"), "should contain bold");
    assert!(content.contains("- bullet one"), "should contain bullet");
    assert!(content.contains("[link text]"), "should contain link");
  }

  #[test]
  fn returns_none_for_schema() {
    let (db, project, file) = load_vault_fixture("evaluate/my_vault", "schemas/Person.tdr");
    let result = export_resource(&db, project, file);
    assert!(result.is_none(), "schema should return None");
  }
}
