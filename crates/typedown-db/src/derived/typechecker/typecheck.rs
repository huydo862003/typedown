//! Tracked query for typechecking

use typedown_macros::query_derived;

use crate::{
  QueryDatabase, TypedownDatabase,
  types::{TdrNode, TypecheckResult},
};

#[query_derived]
pub fn typecheck(db: &TypedownDatabase, node: TdrNode) -> TypecheckResult {
  todo!()
}
