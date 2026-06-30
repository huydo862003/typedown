//! Tracked query to parse a file into an AST

use typedown_macros::query_derived;
use typedown_syntax::{
  green::cache::green_cache,
  parse::ctx::{ParseCtx, ParseResult},
  red::RedNode,
};

use crate::{
  QueryDatabase, TypedownDatabase,
  types::{File, FileAstResult, Project},
};

#[query_derived]
pub fn parse_file(db: &TypedownDatabase, project: Project, file: File) -> FileAstResult {
  let handle = file.handle(db);
  let stream = handle.open().expect("failed to open file");

  let cache = green_cache();
  let mut ctx = ParseCtx::new(stream, cache);
  let ParseResult { diagnostics, ast } = ctx.parse();

  let root = RedNode::new_root(ast.as_node().expect("AST root must be a node").clone());
  FileAstResult::new(
    db,
    file.handle(db),
    project,
    file,
    root,
    diagnostics.to_vec(),
  )
}

#[cfg(test)]
mod tests {
  use std::{collections::HashMap, path::PathBuf, time::SystemTime};

  use typedown_syntax::ast::{AstNode, SourceFile};

  use crate::{
    QueryStorage, TypedownDatabase,
    types::{File, FileHandle, Project},
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

    let project = Project::new(&db, PathBuf::from("/"), HashMap::new());

    let file = File::new(&db, FileHandle::Content(fixture.contents.clone()));
    let result = parse_file(&db, project, file);

    assert!(
      SourceFile::cast(result.ast(&db)).is_some(),
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

    let project = Project::new(&db, PathBuf::from("/"), HashMap::new());
    let file = File::new(
      &db,
      FileHandle::Path(fixture.path.clone(), SystemTime::UNIX_EPOCH),
    );
    let result = parse_file(&db, project, file);

    assert!(
      SourceFile::cast(result.ast(&db)).is_some(),
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

    let project = Project::new(&db, PathBuf::from("/"), HashMap::new());
    let file = File::new(&db, FileHandle::Content(fixture.contents.clone()));
    let result = parse_file(&db, project, file);

    assert!(
      SourceFile::cast(result.ast(&db)).is_some(),
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

    let project = Project::new(&db, PathBuf::from("/"), HashMap::new());
    let file = File::new(
      &db,
      FileHandle::Path(fixture.path.clone(), SystemTime::UNIX_EPOCH),
    );
    let result = parse_file(&db, project, file);

    assert!(
      SourceFile::cast(result.ast(&db)).is_some(),
      "AST should still be a SourceFile even for invalid input"
    );

    let diagnostics = result.diagnostics(&db);
    assert!(
      !diagnostics.is_empty(),
      "Expected diagnostics for missing frontmatter"
    );
  }
}
