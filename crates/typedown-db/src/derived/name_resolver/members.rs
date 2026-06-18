use std::collections::HashMap;

use typedown_macros::query_derived;

use crate::derived::get_vault_config::get_vault_config;
use crate::derived::name_resolver::builtin_scope::builtin_scope;
use crate::derived::name_resolver::file_symbol::file_symbol;
use crate::types::{File, FileHandle, MembersResult, Scope, ScopeKind};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn members(db: &TypedownDatabase, scope: Scope) -> MembersResult {
  match scope.kind(db) {
    ScopeKind::Builtin => MembersResult::new(db, builtin_scope(db).members(db)),
    ScopeKind::File(project, file) => {
      let mut members = HashMap::new();

      if let Some(sym) = file_symbol(db, project, file).value(db) {
        let config = get_vault_config(db, project);
        let _schema_dir = config.schema_dir(db);

        let name = match file.handle(db) {
          FileHandle::Path(path) => path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string(),
          FileHandle::Content(_) => String::new(),
        };

        if !name.is_empty() {
          members.insert(name, sym);
        }
      }

      MembersResult::new(db, members)
    }
    ScopeKind::Project(project) => {
      let config = get_vault_config(db, project);
      let _schema_dir = config.schema_dir(db);
      let handles = project.handles(db);

      let mut members = HashMap::new();

      for (path, handle) in &handles {
        if !path.extension().is_some_and(|ext| ext == "tdr") {
          continue;
        }
        let file = File::new(db, handle.clone());
        if let Some(sym) = file_symbol(db, project, file).value(db) {
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
