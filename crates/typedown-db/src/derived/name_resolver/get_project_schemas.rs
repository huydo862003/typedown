use std::collections::HashMap;

use typedown_macros::query_derived;

use crate::derived::name_resolver::schema_symbol::schema_symbol;
use crate::derived::parse_schemas::parse_schemas;
use crate::types::{File, Project, ProjectSchemaResult};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn get_project_schemas(db: &TypedownDatabase, project: Project) -> ProjectSchemaResult {
  let schemas = parse_schemas(db, project);
  let files = schemas.files(db);

  let mut members = HashMap::new();

  for (path, file_ast) in &files {
    let Some(name) = path.file_stem().and_then(|s| s.to_str()).map(str::to_string) else {
      continue;
    };

    let file = File::new(db, file_ast.handle(db));
    let ast = file_ast.ast(db);
    let symbol = schema_symbol(db, project, file, ast);
    members.insert(name, symbol);
  }

  ProjectSchemaResult::new(db, members)
}
