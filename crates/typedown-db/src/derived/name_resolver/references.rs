use typedown_macros::query_derived;

use crate::types::{References, Symbol};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn references(db: &TypedownDatabase, symbol: Symbol) -> References {
  todo!()
}
