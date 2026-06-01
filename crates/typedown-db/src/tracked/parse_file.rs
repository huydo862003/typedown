//! Tracked query to parse a file into an AST

use std::{cell::RefCell, rc::Rc};

use salsa::Accumulator;

use typedown_syntax::{
  green::{GreenNode, cache::Cache},
  parse::ctx::{ParseCtx, ParseResult},
};

use crate::{Diagnostic, inputs::File};

#[salsa::tracked]
pub fn parse_file(db: &dyn salsa::Database, file: File) -> GreenNode {
  let handle = file.handle(db);
  let stream = handle.open().expect("failed to open file");

  let cache = Rc::new(RefCell::new(Cache::new()));
  let mut ctx = ParseCtx::new(stream, cache);
  let ParseResult { diagnostics, ast } = ctx.parse();

  for diagnostic in diagnostics {
    Diagnostic(diagnostic.clone()).accumulate(db);
  }

  ast
}
