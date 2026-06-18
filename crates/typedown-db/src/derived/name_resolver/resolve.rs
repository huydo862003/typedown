use typedown_macros::query_derived;

use crate::types::{HirValue, Symbol};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn resolve(db: &TypedownDatabase, _hir: HirValue) -> Symbol {
  todo!()
}
