use typedown_macros::query_derived;
use typedown_types::syntax_kind::SyntaxKind;

use crate::derived::name_resolver::file_symbol::MaybeSymbol;
use crate::derived::name_resolver::members::members;
use crate::derived::name_resolver::scope::{parent_scope, scope};
use crate::types::TdrNode;
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn referee(db: &TypedownDatabase, node: TdrNode) -> MaybeSymbol {
  if should_lookup_schema(db, node) {
    schema_referee(db, node)
  } else {
    resource_referee(db, node)
  }
}

/// FIXME: Support tag expressions
fn should_lookup_schema(db: &TypedownDatabase, node: TdrNode) -> bool {
  let red = node.node(db);

  /* Returns true if this node is the value expression of a `_schema:` mapping entry. */
  // Parent must be YamlMappingEntryValue
  let entry_value = match red.parent() {
    Some(parent) if parent.kind() == SyntaxKind::YamlMappingEntryValue => parent,
    _ => return false,
  };
  // Grandparent must be YamlMappingEntry with key "_schema"
  let entry = match entry_value.parent() {
    Some(grandparent) if grandparent.kind() == SyntaxKind::YamlMappingEntry => grandparent,
    _ => return false,
  };
  entry
    .children()
    .any(|child| child.kind() == SyntaxKind::YamlMappingEntryKey && child.text() == "_schema")
}

fn schema_referee(db: &TypedownDatabase, node: TdrNode) -> MaybeSymbol {
  let name = node.node(db).text().trim().to_string();
  let mut current_scope = scope(db, node);
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

fn resource_referee(db: &TypedownDatabase, node: TdrNode) -> MaybeSymbol {
  let name = node.node(db).text().trim().to_string();
  let mut current_scope = scope(db, node);
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
