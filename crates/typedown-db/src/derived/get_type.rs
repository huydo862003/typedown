//! Tracked query to get the type of a green node

use typedown_macros::query_derived;

use crate::{
  QueryDatabase, TypedownDatabase,
  types::{GreenNode, TypeResult},
};

#[query_derived]
pub fn get_type(db: &TypedownDatabase, node: GreenNode) -> TypeResult {
  todo!()
}
