use std::collections::HashMap;

use typedown_macros::query_derived;

use crate::derived::get_vault_config::get_vault_config;
use crate::derived::name_resolver::builtin_scope::{builtin_resource_scope, builtin_schema_scope};
use crate::derived::name_resolver::file_symbol::file_symbol;
use crate::types::{File, FileHandle, MembersResult, Scope, ScopeKind};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn members(db: &TypedownDatabase, scope: Scope) -> MembersResult {
  match scope.kind(db) {
    ScopeKind::Builtin => MembersResult::new(
      db,
      builtin_schema_scope(db).members(db),
      builtin_resource_scope(db).members(db),
    ),
    ScopeKind::File(project, file) => {
      let mut schema_members = HashMap::new();
      let mut resource_members = HashMap::new();

      if let Some(sym) = file_symbol(db, project, file).value(db) {
        let config = get_vault_config(db, project);
        let schema_dir = config.schema_dir(db);

        let (name, is_schema) = match file.handle(db) {
          FileHandle::Path(path) => {
            let name = path
              .file_stem()
              .and_then(|s| s.to_str())
              .unwrap_or_default()
              .to_string();
            let is_schema = path.starts_with(&schema_dir);
            (name, is_schema)
          }
          FileHandle::Content(_) => (String::new(), false),
        };

        if is_schema {
          schema_members.insert(name, sym);
        } else {
          resource_members.insert(name, sym);
        }
      }

      MembersResult::new(db, schema_members, resource_members)
    }
    ScopeKind::Project(project) => {
      let config = get_vault_config(db, project);
      let schema_dir = config.schema_dir(db);
      let handles = project.handles(db);

      let mut schema_members = HashMap::new();
      let mut resource_members = HashMap::new();

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
          if path.starts_with(&schema_dir) {
            schema_members.insert(name, sym);
          } else {
            resource_members.insert(name, sym);
          }
        }
      }

      MembersResult::new(db, schema_members, resource_members)
    }
  }
}
