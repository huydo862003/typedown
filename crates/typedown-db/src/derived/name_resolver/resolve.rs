use typedown_macros::query_derived;
use typedown_syntax::red::RedNode;

use crate::types::{File, Project, Symbol};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn resolve(db: &TypedownDatabase, project: Project, file: File, node: RedNode) -> Symbol {
  todo!()
}
