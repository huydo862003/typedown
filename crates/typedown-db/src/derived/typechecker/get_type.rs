//! Tracked query to get the type of a green node

use typedown_macros::query_derived;

use crate::{
  QueryDatabase, TypedownDatabase,
  types::{File, GreenNode, Project, TypeResult},
};

#[query_derived]
pub fn get_type(db: &TypedownDatabase, project: Project, file: File, node: GreenNode) -> TypeResult {
  todo!()
}
