use typedown_macros::query_derived;
use typedown_types::syntax_kind::SyntaxKind;

use crate::derived::name_resolver::file_symbol::MaybeSymbol;
use crate::derived::name_resolver::members::members;
use crate::derived::name_resolver::scope::{parent_scope, scope};
use crate::types::{HirValue, HirValueKind};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn referee(db: &TypedownDatabase, hir: HirValue) -> MaybeSymbol {
  if should_lookup_schema(db, hir) {
    schema_referee(db, hir)
  } else {
    resource_referee(db, hir)
  }
}

// Whether we should lookup the hir referee as schema
fn should_lookup_schema(db: &TypedownDatabase, hir: HirValue) -> bool {
  // Tag expressions always resolve against the schema scope.
  if matches!(hir.kind(db), HirValueKind::Tag { .. }) {
    return true;
  }

  // Value expression of a `_type:` mapping entry.
  let node = hir.node(db);
  let entry_value = match node.parent() {
    Some(parent) if parent.kind() == SyntaxKind::YamlMappingEntryValue => parent,
    _ => return false,
  };
  let entry = match entry_value.parent() {
    Some(grandparent) if grandparent.kind() == SyntaxKind::YamlMappingEntry => grandparent,
    _ => return false,
  };
  entry
    .children()
    .any(|child| child.kind() == SyntaxKind::YamlMappingEntryKey && child.text() == "_type")
}

/// Lookup in schema namespace
fn schema_referee(db: &TypedownDatabase, hir: HirValue) -> MaybeSymbol {
  let name = match hir.kind(db) {
    HirValueKind::Tag { tag, .. } => tag.node(db).text().trim().to_string(),
    _ => hir.node(db).text().trim().to_string(),
  };

  let mut current_scope = scope(db, hir);
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

/// Lookup in resource namespace
fn resource_referee(db: &TypedownDatabase, hir: HirValue) -> MaybeSymbol {
  let name = hir.node(db).text().trim().to_string();
  let mut current_scope = scope(db, hir);
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
