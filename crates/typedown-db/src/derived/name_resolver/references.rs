use typedown_macros::query_derived;

use crate::types::{Project, References, Symbol};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn references(db: &TypedownDatabase, project: Project, symbol: Symbol) -> References {
  todo!()
}
