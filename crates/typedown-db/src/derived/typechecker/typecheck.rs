//! Tracked query for typechecking

use typedown_macros::query_derived;

use crate::{
  QueryDatabase, TypedownDatabase,
  types::{HirValue, TypecheckResult},
};

#[query_derived]
pub fn typecheck(db: &TypedownDatabase, hir: HirValue) -> TypecheckResult {
  todo!()
}
