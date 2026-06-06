//! Derived query engine for the incremental database

/// A fast id for a derived state
/// Derived id is bound to a database's lifetime
pub trait DerivedId: Clone + Copy + PartialEq + Eq + std::hash::Hash {
  /// Marker used by macros to verify a type implements DerivedId at compile time.
  #[cfg(debug_assertions)]
  #[doc(hidden)]
  const __TYPEDOWN_DERIVED_ID: () = ();
}

#[cfg(test)]
mod tests {
  use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::OnceLock};

  use typedown_macros::{query_db, query_derived, query_input};
  use typedown_syntax::{
    ast::{AstNode, SourceFile},
    green::{GreenNode, cache::Cache},
    parse::ctx::ParseCtx,
    red::RedNode,
  };
  use typedown_types::{
    diagnostic::{self, Diagnostic},
    string_stream::StringStream,
  };

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
}
