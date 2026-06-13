use std::collections::HashMap;

use typedown_macros::query_derived;

use crate::types::{File, Project, TdrNode};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SymbolKind {
  Schema,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScopeKind {
  Project(Project),
  File(Project, File),
}

#[query_derived]
pub struct Scope {
  #[id]
  kind: ScopeKind,
}

impl Scope {
  pub fn project_scope(db: &(impl crate::QueryDatabase + ?Sized), project: Project) -> Self {
    Self::new(db, ScopeKind::Project(project))
  }

  pub fn file_scope(db: &(impl crate::QueryDatabase + ?Sized), project: Project, file: File) -> Self {
    Self::new(db, ScopeKind::File(project, file))
  }
}

#[query_derived]
pub struct Symbol {
  #[id]
  node: TdrNode,
  kind: SymbolKind,
}

#[query_derived]
pub struct References {
  nodes: Vec<TdrNode>,
}

#[query_derived]
pub struct ProjectSchemaResult {
  members: HashMap<String, Symbol>,
}

#[query_derived]
pub struct MembersResult {
  schema_members: HashMap<String, Symbol>,
  resource_members: HashMap<String, Symbol>,
}
