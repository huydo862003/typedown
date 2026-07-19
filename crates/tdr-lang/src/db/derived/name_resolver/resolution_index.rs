//! Per-file index mapping each symbol to the HIR nodes that reference it

use tdr_macros::query_derived;

use crate::db::TypedownDatabase;
use crate::db::derived::name_resolver::referee::referee;
use crate::db::types::{File, HirValue, HirValueKind, InterpolatedPart, Project, Symbol};
use crate::db::utils::lower_file;
use tdr_incremental::QueryDatabase;

#[query_derived]
pub struct ResolutionIndex {
  // Flat list of (symbol, hir) pairs for encoding compatibility
  entries: Vec<(Symbol, HirValue)>,
}

impl ResolutionIndex {
  /// Look up all HIR nodes that reference a given symbol
  pub fn get_references(&self, db: &TypedownDatabase, symbol: Symbol) -> Vec<HirValue> {
    self
      .entries(db)
      .iter()
      .filter(|(sym, _)| *sym == symbol)
      .map(|(_, hir)| *hir)
      .collect()
  }
}

/// Build an index of all symbol references in a file
#[query_derived]
pub fn resolution_index(db: &TypedownDatabase, project: Project, file: File) -> ResolutionIndex {
  let mut entries = vec![];
  let (hir, _) = lower_file(db, project, file);
  if let Some(hir) = hir {
    collect_references(db, hir, &mut entries);
  }
  ResolutionIndex::new(db, entries)
}

fn collect_references(db: &TypedownDatabase, hir: HirValue, entries: &mut Vec<(Symbol, HirValue)>) {
  match hir.kind(db) {
    HirValueKind::Mapping(values) => {
      for (_, value) in values {
        collect_references(db, value, entries);
      }
    }
    HirValueKind::Sequence(items) => {
      for item in items {
        collect_references(db, item, entries);
      }
    }
    HirValueKind::Interpolated(parts) | HirValueKind::Markdown(parts) => {
      for part in parts {
        if let InterpolatedPart::Expr(expr) = part {
          collect_references(db, expr, entries);
        }
      }
    }
    HirValueKind::Tag { tag, inner } => {
      collect_references(db, *tag, entries);
      collect_references(db, *inner, entries);
    }
    HirValueKind::Unary { operand, .. } => {
      collect_references(db, *operand, entries);
    }
    HirValueKind::Binary { left, right, .. } => {
      collect_references(db, *left, entries);
      collect_references(db, *right, entries);
    }
    HirValueKind::Call { callee, args } => {
      // Call nodes can resolve (e.g. fref)
      if let Some(symbol) = referee(db, hir).value(db) {
        entries.push((symbol, hir));
      }
      collect_references(db, *callee, entries);
      for arg in args {
        collect_references(db, arg, entries);
      }
    }
    HirValueKind::Index { expr, indices } => {
      collect_references(db, *expr, entries);
      for idx in indices {
        collect_references(db, idx, entries);
      }
    }
    // Only Ident nodes resolve to symbols via referee
    HirValueKind::Ident(_) => {
      if let Some(symbol) = referee(db, hir).value(db) {
        entries.push((symbol, hir));
      }
    }
    HirValueKind::Str(_)
    | HirValueKind::Num(_)
    | HirValueKind::Math(_)
    | HirValueKind::Bool(_)
    | HirValueKind::Null => {}
  }
}

/// Find all HIR nodes across the project that reference a given symbol
#[query_derived]
pub fn references(db: &TypedownDatabase, project: Project, symbol: Symbol) -> ReferencesResult {
  let mut refs = vec![];
  for file in project.files(db).values() {
    let idx = resolution_index(db, project, *file);
    refs.extend(idx.get_references(db, symbol));
  }
  ReferencesResult::new(db, refs)
}

#[query_derived]
pub struct ReferencesResult {
  references: Vec<HirValue>,
}

#[cfg(test)]
mod tests {
  use super::{references, resolution_index};
  use crate::db::derived::name_resolver::file_symbol::file_symbol;
  use crate::db::fixtures::load_vault_fixture;

  // resolution_index finds schema references in a typed content file
  #[test]
  fn resolution_index_finds_type_reference() {
    let (db, project, file) = load_vault_fixture("typecheck/my_vault", "content/valid_person.tdr");
    let idx = resolution_index(&db, project, file);
    // valid_person.tdr has _type: Person, so "Person" should be in the index
    assert!(
      !idx.entries(&db).is_empty(),
      "resolution index should have entries for a typed file"
    );
  }

  // resolution_index returns empty for a file with no references
  #[test]
  fn resolution_index_empty_for_untyped() {
    let (db, project, file) = load_vault_fixture("typecheck/my_vault", "content/literal_value.tdr");
    let idx = resolution_index(&db, project, file);
    assert!(
      idx.entries(&db).is_empty(),
      "untyped file with no identifiers should have empty index"
    );
  }

  // references finds all files that reference a given schema symbol
  #[test]
  fn references_finds_usages_across_files() {
    let (db, project, _) = load_vault_fixture("typecheck/my_vault", "content/valid_person.tdr");
    // Get the Person schema symbol
    let schema_files = project.files(&db);
    let person_path = schema_files
      .keys()
      .find(|path| path.to_string_lossy().contains("Person.tdr"))
      .expect("should have Person.tdr");
    let person_file = schema_files[person_path];
    let person_symbol = file_symbol(&db, project, person_file)
      .value(&db)
      .expect("Person.tdr should have a symbol");

    let result = references(&db, project, person_symbol);
    // valid_person.tdr references Person via _type
    assert!(
      !result.references(&db).is_empty(),
      "Person should have at least one reference"
    );
  }

  // resolution_index finds fref references
  #[test]
  fn resolution_index_finds_fref() {
    let (db, project, file) =
      load_vault_fixture("typecheck/narrow_vault", "content/article_fref_status.tdr");
    let idx = resolution_index(&db, project, file);
    // article_fref_status.tdr has fref("content/summary.tdr") which resolves to a symbol
    let entries = idx.entries(&db);
    assert!(
      entries.len() >= 2,
      "should have at least 2 references: Article schema + fref target, got {}",
      entries.len()
    );
  }

  // get_references filters by symbol
  #[test]
  fn get_references_filters_correctly() {
    let (db, project, file) =
      load_vault_fixture("typecheck/narrow_vault", "content/article_fref_status.tdr");
    let idx = resolution_index(&db, project, file);

    // Get the Article schema symbol
    let schema_files = project.files(&db);
    let article_path = schema_files
      .keys()
      .find(|path| path.to_string_lossy().contains("Article.tdr"))
      .expect("should have Article.tdr");
    let article_file = schema_files[article_path];
    let article_symbol = file_symbol(&db, project, article_file)
      .value(&db)
      .expect("Article.tdr should have a symbol");

    let article_refs = idx.get_references(&db, article_symbol);
    assert!(
      !article_refs.is_empty(),
      "should find Article reference in the file"
    );

    // Fref references the content file content/summary.tdr
    let summary_path = schema_files
      .keys()
      .find(|path| path.to_string_lossy().contains("content/summary.tdr"))
      .expect("should have content/summary.tdr");
    let summary_file = schema_files[summary_path];
    let summary_symbol = file_symbol(&db, project, summary_file)
      .value(&db)
      .expect("summary.tdr should have a symbol");

    let summary_refs = idx.get_references(&db, summary_symbol);
    assert!(
      !summary_refs.is_empty(),
      "should find summary reference via fref"
    );
  }
}
