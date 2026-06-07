//! Tracked query to parse a file into an AST

use std::{cell::RefCell, rc::Rc};

use typedown_macros::query_derived;
use typedown_syntax::{
  green::{GreenNode, cache::Cache},
  parse::ctx::{ParseCtx, ParseResult},
};
use typedown_types::diagnostic::Diagnostic;

use crate::{
  QueryDatabase, TypedownDatabase,
  inputs::{File, FileHandle},
};

#[query_derived]
struct FileAst {
  handle: FileHandle,
  ast: GreenNode,
  diagnostics: Vec<Diagnostic>,
}

#[query_derived]
pub fn parse_file(db: &TypedownDatabase, file: File) -> FileAst {
  let handle = file.handle(db);
  let stream = handle.open().expect("failed to open file");

  let cache = Rc::new(RefCell::new(Cache::new()));
  let mut ctx = ParseCtx::new(stream, cache);
  let ParseResult { diagnostics, ast } = ctx.parse();

  FileAst::new(db, file.handle(db), ast, diagnostics.to_vec())
}
