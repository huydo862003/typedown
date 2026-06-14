use typedown_macros::query_derived;

use crate::types::{File, Project, Scope, ScopeKind, TdrNode};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub struct MaybeScope {
  pub value: Option<Scope>,
}

#[query_derived]
pub fn scope(db: &TypedownDatabase, project: Project, file: File, node: TdrNode) -> Scope {
  Scope::file_scope(db, project, file)
}

#[query_derived]
pub fn parent_scope(db: &TypedownDatabase, scope: Scope) -> MaybeScope {
  match scope.kind(db) {
    ScopeKind::Project(_) => MaybeScope::new(db, None),
    ScopeKind::File(project, _) => MaybeScope::new(db, Some(Scope::project_scope(db, project))),
  }
}
