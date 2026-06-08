//! Tracked query to parse a file into an AST

use std::{cell::RefCell, rc::Rc};

use typedown_macros::query_derived;
use typedown_syntax::{
  green::cache::Cache,
  parse::ctx::{ParseCtx, ParseResult},
};

use crate::{
  QueryDatabase, TypedownDatabase,
  types::{File, FileAstResult, GreenNode},
};

#[query_derived]
pub fn parse_file(db: &TypedownDatabase, file: File) -> FileAstResult {
  let handle = file.handle(db);
  let stream = handle.open().expect("failed to open file");

  let cache = Rc::new(RefCell::new(Cache::new()));
  let mut ctx = ParseCtx::new(stream, cache);
  let ParseResult { diagnostics, ast } = ctx.parse();

  let ast = GreenNode::new(db, ast);
  FileAstResult::new(db, file.handle(db), ast, diagnostics.to_vec())
}

#[cfg(test)]
mod tests {
  use typedown_syntax::ast::SourceFile;

  use crate::{
    QueryStorage, TypedownDatabase,
    inputs::{File, FileHandle},
  };

  use super::parse_file;

  #[test]
  fn parse_file_with_content_handle() {
    let fixtures = crate::fixtures::load_fixtures("parse_file");
    let fixture = fixtures
      .get("valid.tdr")
      .expect("missing valid.tdr fixture");

    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };

    let file = File::new(&db, FileHandle::Content(fixture.contents.clone()));
    let result = parse_file(&db, file);

    assert!(
      result.ast(&db).try_cast::<SourceFile>(&db).is_some(),
      "AST root should be a SourceFile"
    );

    let diagnostics = result.diagnostics(&db);
    assert!(
      diagnostics.is_empty(),
      "Expected no diagnostics, got: {:?}",
      diagnostics
    );
  }

  #[test]
  fn parse_file_with_path_handle() {
    let fixtures = crate::fixtures::load_fixtures("parse_file");
    let fixture = fixtures
      .get("valid.tdr")
      .expect("missing valid.tdr fixture");

    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };

    let file = File::new(&db, FileHandle::Path(fixture.path.clone()));
    let result = parse_file(&db, file);

    assert!(
      result.ast(&db).try_cast::<SourceFile>(&db).is_some(),
      "AST root should be a SourceFile"
    );

    let diagnostics = result.diagnostics(&db);
    assert!(
      diagnostics.is_empty(),
      "Expected no diagnostics, got: {:?}",
      diagnostics
    );
  }

  #[test]
  fn parse_invalid_file_with_content_handle() {
    let fixtures = crate::fixtures::load_fixtures("parse_file");
    let fixture = fixtures
      .get("invalid.tdr")
      .expect("missing invalid.tdr fixture");

    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };

    let file = File::new(&db, FileHandle::Content(fixture.contents.clone()));
    let result = parse_file(&db, file);

    assert!(
      result.ast(&db).try_cast::<SourceFile>(&db).is_some(),
      "AST should still be a SourceFile even for invalid input"
    );

    let diagnostics = result.diagnostics(&db);
    assert!(
      !diagnostics.is_empty(),
      "Expected diagnostics for missing frontmatter"
    );
  }

  #[test]
  fn parse_invalid_file_with_path_handle() {
    let fixtures = crate::fixtures::load_fixtures("parse_file");
    let fixture = fixtures
      .get("invalid.tdr")
      .expect("missing invalid.tdr fixture");

    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };

    let file = File::new(&db, FileHandle::Path(fixture.path.clone()));
    let result = parse_file(&db, file);

    assert!(
      result.ast(&db).try_cast::<SourceFile>(&db).is_some(),
      "AST should still be a SourceFile even for invalid input"
    );

    let diagnostics = result.diagnostics(&db);
    assert!(
      !diagnostics.is_empty(),
      "Expected diagnostics for missing frontmatter"
    );
  }
}
