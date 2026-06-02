//! Tracked query to parse a file into an AST

use std::{cell::RefCell, rc::Rc};

use typedown_syntax::{
  green::{GreenNode, cache::Cache},
  parse::ctx::{ParseCtx, ParseResult},
};

use crate::{TypedownDatabase, inputs::File};

pub fn parse_file(db: TypedownDatabase, file: File) -> GreenNode {
  let handle = file.handle(db);
  let stream = handle.open().expect("failed to open file");

  let cache = Rc::new(RefCell::new(Cache::new()));
  let mut ctx = ParseCtx::new(stream, cache);
  let ParseResult { diagnostics, ast } = ctx.parse();

  ast
}
