use typedown_macros::query_derived;

use crate::types::{File, Project, Scope, ScopeKind, TdrNode};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn scope(db: &TypedownDatabase, project: Project, file: File, node: TdrNode) -> Scope {
  Scope::file_scope(db, project, file)
}

#[query_derived]
pub fn parent_scope(db: &TypedownDatabase, scope: Scope) -> Option<Scope> {
  match scope.kind(db) {
    ScopeKind::Project(_) => None,
    ScopeKind::File(project, _) => Some(Scope::project_scope(db, project)),
  }
}
