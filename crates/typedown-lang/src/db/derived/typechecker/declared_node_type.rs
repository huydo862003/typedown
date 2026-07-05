//! Tracked query to get the declared (top-down) type of a HIR value.

use crate::db::TypedownDatabase;
use crate::db::derived::evaluate::evaluate_type::evaluate_type;
use crate::db::derived::name_resolver::members::members;
use crate::db::types::{HirValue, Scope, TypeMemberResult};
use crate::red::RedNode;
use typedown_incremental::QueryDatabase;
use typedown_macros::query_derived;
use typedown_types::syntax_kind::SyntaxKind;
#[query_derived]
#[query_derived]
pub fn declared_node_type(db: &TypedownDatabase, hir: HirValue) -> TypeMemberResult {
  let project = hir.project(db);
  let node = hir.node(db);

  // Walk up to the enclosing mapping entry to find the field name
  let entry = match find_ancestor(&node, SyntaxKind::YamlMappingEntry) {
    Some(entry) => entry,
    None => return TypeMemberResult::new(db, None, vec![]),
  };

  let field_name = match entry
    .children()
    .find(|child| child.kind() == SyntaxKind::YamlMappingEntryKey)
  {
    Some(key) => key.text().trim().to_string(),
    None => return TypeMemberResult::new(db, None, vec![]),
  };

  // Walk up to the enclosing mapping and find the _type field
  let mapping = match find_ancestor(&entry, SyntaxKind::YamlMapping) {
    Some(mapping) => mapping,
    None => return TypeMemberResult::new(db, None, vec![]),
  };

  let schema_name = match schema_name_in_mapping(&mapping) {
    Some(name) => name,
    None => return TypeMemberResult::new(db, None, vec![]),
  };

  // Resolve the schema symbol and get its type
  let scope = Scope::project_scope(db, project);
  let schema_symbol = match members(db, scope).members(db).get(&schema_name).copied() {
    Some(sym) => sym,
    None => return TypeMemberResult::new(db, None, vec![]),
  };

  let schema_type = match evaluate_type(db, schema_symbol).typ(db) {
    Some(typ) => typ,
    None => return TypeMemberResult::new(db, None, vec![]),
  };

  // Downcast to a product type and look up the field
  let Some(product) = schema_type.as_tdr_product_type() else {
    return TypeMemberResult::new(db, None, vec![]);
  };

  // Return the full TypeMember (preserves descriptors like OPTIONAL)
  let field_member = match product.fields(db).get(&field_name) {
    Some(member) => *member,
    None => return TypeMemberResult::new(db, None, vec![]),
  };

  TypeMemberResult::new(db, Some(field_member), vec![])
}

#[cfg(test)]
mod tests {
  use crate::db::types::TdrTypeLike;
  use crate::db::{
    TypedownDatabase,
    derived::typechecker::declared_node_type::declared_node_type,
    fixtures::load_vault_fixture,
    types::{File, HirValueKind, MemberType, Project},
    utils::lower_file,
  };

  fn get_field_hir(
    db: &TypedownDatabase,
    project: Project,
    file: File,
    field: &str,
  ) -> Option<crate::db::types::HirValue> {
    let (hir, _) = lower_file(db, project, file);
    let hir = hir?;
    if let HirValueKind::Mapping(entries) = hir.kind(db) {
      entries
        .into_iter()
        .find(|(key, _)| key == field)
        .map(|(_, value)| value)
    } else {
      None
    }
  }

  // declared type for a known field returns the schema member
  #[test]
  fn declared_node_type_known_field_returns_member() {
    let (db, project, file) = load_vault_fixture("typecheck/my_vault", "content/valid_person.tdr");
    let name_hir = get_field_hir(&db, project, file, "name")
      .expect("valid_person.tdr should have a 'name' field");

    let result = declared_node_type(&db, name_hir);

    assert!(
      result.diagnostics(&db).is_empty(),
      "expected no diagnostics, got: {:?}",
      result.diagnostics(&db)
    );
    let member = result
      .member(&db)
      .expect("'name' field should have a declared TypeMember");
    // Person schema declares name: string
    match member.typ(&db) {
      MemberType::Simple(typ) => assert_eq!(
        typ.display_name(&db),
        "string",
        "expected declared type 'string', got '{}'",
        typ.display_name(&db)
      ),
      _other => panic!("expected Simple member type"),
    }
  }

  // declared type for a field not in any schema returns None
  #[test]
  fn declared_node_type_untyped_mapping_returns_none() {
    // literal_value.tdr has no _type field, so no schema to look up
    let (db, project, file) = load_vault_fixture("typecheck/my_vault", "content/literal_value.tdr");
    let (hir, _) = lower_file(&db, project, file);
    let hir = hir.expect("literal_value.tdr should have parseable frontmatter");

    let result = declared_node_type(&db, hir);

    assert!(
      result.member(&db).is_none(),
      "untyped mapping root should have no declared member"
    );
  }
}

/// Walk up the red tree to find the nearest ancestor with the given kind.
fn find_ancestor(node: &RedNode, kind: SyntaxKind) -> Option<RedNode> {
  let mut current = node.parent()?;
  loop {
    if current.kind() == kind {
      return Some(current);
    }
    current = current.parent()?;
  }
}

/// Find the value of the `_type` field in a mapping node.
fn schema_name_in_mapping(mapping: &RedNode) -> Option<String> {
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
