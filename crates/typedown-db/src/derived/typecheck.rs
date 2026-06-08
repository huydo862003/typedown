//! Tracked query for typechecking

use typedown_macros::query_derived;

use crate::{
  QueryDatabase, TypedownDatabase,
  types::{GreenNode, TypecheckResult},
};

#[query_derived]
pub fn typecheck(db: &TypedownDatabase, node: GreenNode) -> TypecheckResult {
  todo!()
}
