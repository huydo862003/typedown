use typedown_macros::query_derived;

use crate::types::{GreenNode, Symbol};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn symbol(db: &TypedownDatabase, node: GreenNode) -> Symbol {
  todo!()
}
