use typedown_macros::query_derived;

use crate::{QueryDatabase, TypedownDatabase};
use crate::types::{GreenNode, Symbol};

#[query_derived]
pub fn resolve(db: &TypedownDatabase, node: GreenNode) -> Symbol {
  todo!()
}
