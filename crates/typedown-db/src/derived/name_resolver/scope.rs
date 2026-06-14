use typedown_macros::query_derived;

use crate::types::{Scope, ScopeKind, TdrNode};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub struct MaybeScope {
  pub value: Option<Scope>,
}

#[query_derived]
pub fn scope(db: &TypedownDatabase, node: TdrNode) -> Scope {
  let project = node.project(db);
  let file = node.file(db);
  Scope::file_scope(db, project, file)
}

#[query_derived]
pub fn parent_scope(db: &TypedownDatabase, scope: Scope) -> MaybeScope {
  match scope.kind(db) {
    ScopeKind::Project(_) => MaybeScope::new(db, None),
    ScopeKind::File(project, _) => MaybeScope::new(db, Some(Scope::project_scope(db, project))),
  }
}
