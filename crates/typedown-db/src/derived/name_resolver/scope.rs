use typedown_macros::query_derived;
use typedown_syntax::red::RedNode;

use crate::types::{File, Project, Scope, ScopeKind};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub struct MaybeScope {
  pub value: Option<Scope>,
}

#[query_derived]
pub fn scope(db: &TypedownDatabase, project: Project, file: File, node: RedNode) -> Scope {
  Scope::file_scope(db, project, file)
}

#[query_derived]
pub fn parent_scope(db: &TypedownDatabase, scope: Scope) -> MaybeScope {
  match scope.kind(db) {
    ScopeKind::Builtin => MaybeScope::new(db, None),
    ScopeKind::Project(_) => MaybeScope::new(db, Some(Scope::builtin_scope(db))),
    ScopeKind::File(project, _) => MaybeScope::new(db, Some(Scope::project_scope(db, project))),
  }
}
