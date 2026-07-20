use std::collections::HashMap;

use tdr_macros::query_derived;

use crate::db::TypedownDatabase;
use crate::db::derived::name_resolver::builtin_scope::builtin_scope;
use crate::db::derived::name_resolver::file_symbol::file_symbol;
use crate::db::types::{MembersResult, Scope, ScopeKind};
use tdr_incremental::QueryDatabase;

#[query_derived]
pub fn members(db: &TypedownDatabase, scope: Scope) -> MembersResult {
  match scope.kind(db) {
    ScopeKind::Builtin => MembersResult::new(db, builtin_scope(db).members(db)),
    ScopeKind::File(project, file) => {
      let mut members = HashMap::new();

      if let Some(sym) = file_symbol(db, project, file).value(db) {
        let name = file
          .handle(db)
          .path()
          .and_then(|p| p.file_stem())
          .and_then(|s| s.to_str())
          .unwrap_or_default()
          .to_string();

        if !name.is_empty() {
          members.insert(name, sym);
        }
      }

      MembersResult::new(db, members)
    }
    ScopeKind::Project(project) => {
      let proj_files = project.files(db);

      let mut members = HashMap::new();

      for (path, file) in &proj_files {
        if !path.extension().is_some_and(|ext| ext == "tdr") {
          continue;
        }
        if let Some(sym) = file_symbol(db, project, *file).value(db) {
          let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
          members.insert(name, sym);
        }
      }

      MembersResult::new(db, members)
    }
  }
}
