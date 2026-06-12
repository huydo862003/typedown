use typedown_macros::query_derived;

use crate::types::{MembersResult, Project};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn project_members(db: &TypedownDatabase, project: Project) -> MembersResult {
  todo!()
}
