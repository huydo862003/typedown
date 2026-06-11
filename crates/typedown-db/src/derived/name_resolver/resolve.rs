use typedown_macros::query_derived;

use crate::types::{File, GreenNode, Project, Symbol};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn resolve(db: &TypedownDatabase, project: Project, file: File, node: GreenNode) -> Symbol {
  todo!()
}
