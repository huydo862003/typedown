//! Tracked query for typechecking

use typedown_macros::query_derived;
use typedown_syntax::red::RedNode;

use crate::{
  QueryDatabase, TypedownDatabase,
  types::{File, Project, TypecheckResult},
};

#[query_derived]
pub fn typecheck(
  db: &TypedownDatabase,
  project: Project,
  file: File,
  node: RedNode,
) -> TypecheckResult {
  todo!()
}
