//! Tracked query for typechecking

use typedown_macros::query_derived;

use crate::{
  QueryDatabase, TypedownDatabase,
  types::{File, GreenNode, Project, TypecheckResult},
};

#[query_derived]
pub fn typecheck(db: &TypedownDatabase, project: Project, file: File, node: GreenNode) -> TypecheckResult {
  todo!()
}
