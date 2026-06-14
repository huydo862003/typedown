//! Derived query engine for the incremental database

/// A fast id for a derived state
/// Derived id is bound to a database's lifetime
pub trait DerivedId: super::id::Id + From<usize> + Into<usize> {
  /// Marker used by macros to verify a type implements DerivedId at compile time.
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  const __TYPEDOWN_DERIVED_ID: () = ();
}

#[cfg(test)]
mod tests {
  use std::{cell::RefCell, path::PathBuf, rc::Rc};

  use typedown_macros::{query_db, query_derived, query_input};
  use typedown_syntax::{
    ast::{AstNode, SourceFile},
    green::{cache::Cache, GreenNode},
    parse::ctx::ParseCtx,
    red::RedNode,
  };
  use typedown_types::{diagnostic::Diagnostic, string_stream::StringStream};

  use crate::{QueryDatabase, QueryStorage};

  #[query_db]
  struct Database {
    storage: QueryStorage,
  }

  #[query_input]
  struct ProgramFile {
    path: PathBuf,
    source: String,
  }

  #[query_derived]
  struct ParseResult {
    #[id]
    path: PathBuf,
    ast: GreenNode,
    diagnostics: Vec<Diagnostic>,
  }

  thread_local! {
    static CACHE: Rc<RefCell<Cache>> = Rc::new(RefCell::new(Cache::new()));
  }

  #[query_derived]
  fn parse_file(db: &Database, file: ProgramFile) -> ParseResult {
    let cache = CACHE.with(|cache| cache.clone());
    let source = file.source(db);
    let stream = StringStream::new(&source);
    let mut parser = ParseCtx::new(stream, cache);
    let typedown_syntax::parse::ctx::ParseResult { ast, diagnostics } = parser.parse();
    ParseResult::new(db, file.path(db), ast, diagnostics.to_vec())
  }

  #[test]
  fn derived_query_returns_correct_value() {
    let db = Database {
      storage: QueryStorage::default(),
    };

    let file = ProgramFile::new(
      &db,
      PathBuf::from("/test.td"),
      String::from("---\n---\n# Hello World\n"),
    );

    let result = parse_file(&db, file);

    // Verify the path is preserved
    assert_eq!(result.path(&db), PathBuf::from("/test.td"));

    // Verify the AST was parsed as a SourceFile
    let ast = result.ast(&db);
    let node = ast.as_node().expect("AST should be a node, not a token");
    let red = RedNode::new_root(node.clone());
    assert!(
      SourceFile::cast(red).is_some(),
      "AST root should be a SourceFile"
    );

    // Verify no diagnostics for valid input
    let diagnostics = result.diagnostics(&db);
    assert!(
      diagnostics.is_empty(),
      "Expected no diagnostics, got: {:?}",
      diagnostics
    );
  }

  #[test]
  fn derived_query_rerun_without_change_does_not_crash() {
    let db = Database {
      storage: QueryStorage::default(),
    };

    let file = ProgramFile::new(
      &db,
      PathBuf::from("/test.td"),
      String::from("---\n---\n# Hello World\n"),
    );

    let rev_before = db
      .storage
      .revision
      .load(std::sync::atomic::Ordering::Acquire);

    let result1 = parse_file(&db, file);
    let result2 = parse_file(&db, file);

    assert_eq!(result1, result2);

    // Derived query execution should not bump the revision
    let rev_after = db
      .storage
      .revision
      .load(std::sync::atomic::Ordering::Acquire);
    assert_eq!(
      rev_before, rev_after,
      "revision should not bump from derived query execution"
    );
  }

  #[test]
  fn derived_query_on_two_inputs() {
    let db = Database {
      storage: QueryStorage::default(),
    };

    let file1 = ProgramFile::new(
      &db,
      PathBuf::from("/a.td"),
      String::from("---\n---\n# First\n"),
    );
    let file2 = ProgramFile::new(
      &db,
      PathBuf::from("/b.td"),
      String::from("---\n---\n# Second\n"),
    );

    let rev_before = db
      .storage
      .revision
      .load(std::sync::atomic::Ordering::Acquire);

    let result1 = parse_file(&db, file1);
    let result2 = parse_file(&db, file2);

    let rev_after = db
      .storage
      .revision
      .load(std::sync::atomic::Ordering::Acquire);
    assert_eq!(
      rev_before, rev_after,
      "revision should not bump from derived query execution"
    );

    // Different inputs should produce different results
    assert_ne!(result1, result2);

    // Each result should have the correct path
    assert_eq!(result1.path(&db), PathBuf::from("/a.td"));
    assert_eq!(result2.path(&db), PathBuf::from("/b.td"));

    // Both should parse without diagnostics
    assert!(result1.diagnostics(&db).is_empty());
    assert!(result2.diagnostics(&db).is_empty());

    // Rerunning should return the same cached results
    let result1_again = parse_file(&db, file1);
    let result2_again = parse_file(&db, file2);
    assert_eq!(result1, result1_again);
    assert_eq!(result2, result2_again);
  }
}
