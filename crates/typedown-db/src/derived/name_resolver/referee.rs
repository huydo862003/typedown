use typedown_macros::query_derived;

use crate::types::{File, Project, Symbol, TdrNode};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn referee(db: &TypedownDatabase, project: Project, file: File, node: TdrNode) -> Symbol {
  todo!()
}
