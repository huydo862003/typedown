use typedown_macros::query_derived;

use crate::types::{Symbol, TdrNode};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn resolve(db: &TypedownDatabase, node: TdrNode) -> Symbol {
  todo!()
}
