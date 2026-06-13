use typedown_macros::query_derived;

use crate::types::{File, TdrNode, Project, Symbol};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn referee(db: &TypedownDatabase, project: Project, file: File, node: TdrNode) -> Symbol {
  todo!()
}
