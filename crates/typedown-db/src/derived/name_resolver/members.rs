use typedown_macros::query_derived;

use crate::types::{File, TdrNode, MembersResult, Project};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn members(
  db: &TypedownDatabase,
  project: Project,
  file: File,
  node: TdrNode,
) -> MembersResult {
  todo!()
}
