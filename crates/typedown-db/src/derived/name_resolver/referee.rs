use typedown_macros::query_derived;
use typedown_syntax::red::RedNode;
use typedown_types::syntax_kind::SyntaxKind;

use crate::derived::name_resolver::file_symbol::MaybeSymbol;
use crate::derived::name_resolver::members::members;
use crate::derived::name_resolver::scope::{parent_scope, scope};
use crate::types::{File, Project};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn referee(db: &TypedownDatabase, project: Project, file: File, node: RedNode) -> MaybeSymbol {
  if should_lookup_schema(db, node.clone()) {
    schema_referee(db, project, file, node)
  } else {
    resource_referee(db, project, file, node)
  }
}

/// FIXME: Support tag expressions
fn should_lookup_schema(db: &TypedownDatabase, node: RedNode) -> bool {
  /* Returns true if this node is the value expression of a `_type:` mapping entry. */
  // Parent must be YamlMappingEntryValue
  let entry_value = match node.parent() {
    Some(parent) if parent.kind() == SyntaxKind::YamlMappingEntryValue => parent,
    _ => return false,
  };
  // Grandparent must be YamlMappingEntry with key "_type"
  let entry = match entry_value.parent() {
    Some(grandparent) if grandparent.kind() == SyntaxKind::YamlMappingEntry => grandparent,
    _ => return false,
  };
  entry
    .children()
    .any(|child| child.kind() == SyntaxKind::YamlMappingEntryKey && child.text() == "_type")
}

fn schema_referee(db: &TypedownDatabase, project: Project, file: File, node: RedNode) -> MaybeSymbol {
  let name = node.text().trim().to_string();
  let mut current_scope = scope(db, project, file, node);
  loop {
    let result = members(db, current_scope);
    if let Some(sym) = result.schema_members(db).get(&name) {
      return MaybeSymbol::new(db, Some(*sym));
    }
    match parent_scope(db, current_scope).value(db) {
      Some(parent) => current_scope = parent,
      None => return MaybeSymbol::new(db, None),
    }
  }
}

fn resource_referee(db: &TypedownDatabase, project: Project, file: File, node: RedNode) -> MaybeSymbol {
  let name = node.text().trim().to_string();
  let mut current_scope = scope(db, project, file, node);
  loop {
    let result = members(db, current_scope);
    if let Some(sym) = result.resource_members(db).get(&name) {
      return MaybeSymbol::new(db, Some(*sym));
    }
    match parent_scope(db, current_scope).value(db) {
      Some(parent) => current_scope = parent,
      None => return MaybeSymbol::new(db, None),
    }
  }
}
