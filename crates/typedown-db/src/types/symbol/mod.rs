use std::collections::HashMap;

use typedown_macros::query_derived;

use crate::types::{File, Project};
use typedown_syntax::red::RedNode;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SymbolKind {
  UserDefinedSchema(File),
  BuiltinSchema(BuiltinSchemaKind),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BuiltinSchemaKind {
  Schema,
  Str,
  Num,
  Bool,
  Date,
  DateTime,
  Time,
  List,
  Dict,
  Link,
}

impl SymbolKind {
  pub fn is_schema(&self) -> bool {
    matches!(
      self,
      SymbolKind::UserDefinedSchema(_) | SymbolKind::BuiltinSchema(_)
    )
  }

  pub fn is_resource(&self) -> bool {
    !self.is_schema()
  }

  pub fn is_user_defined(&self) -> bool {
    matches!(self, SymbolKind::UserDefinedSchema(_))
  }

  pub fn is_builtin(&self) -> bool {
    matches!(self, SymbolKind::BuiltinSchema(_))
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScopeKind {
  Builtin,
  Project(Project),
  File(Project, File),
}

#[query_derived]
pub struct Scope {
  #[id]
  kind: ScopeKind,
}

impl Scope {
  pub fn builtin_scope(db: &(impl crate::QueryDatabase + ?Sized)) -> Self {
    Self::new(db, ScopeKind::Builtin)
  }

  pub fn project_scope(db: &(impl crate::QueryDatabase + ?Sized), project: Project) -> Self {
    Self::new(db, ScopeKind::Project(project))
  }

  pub fn file_scope(
    db: &(impl crate::QueryDatabase + ?Sized),
    project: Project,
    file: File,
  ) -> Self {
    Self::new(db, ScopeKind::File(project, file))
  }
}

#[query_derived]
pub struct Symbol {
  #[id]
  kind: SymbolKind,
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
