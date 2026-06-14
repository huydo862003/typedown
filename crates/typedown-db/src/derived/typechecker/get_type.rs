//! Tracked query to get the type of a green node

use typedown_macros::query_derived;
use typedown_syntax::ast::{AstNode, YamlMapping};
use typedown_syntax::red::RedNode;
use typedown_types::diagnostic::Diagnostic;
use typedown_types::syntax_kind::SyntaxKind;

use crate::derived::evaluate::evaluate_schema::evaluate_schema;
use crate::derived::name_resolver::referee::referee;
use crate::types::{File, Project, TdrObjectType, TypeResult};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn get_type(db: &TypedownDatabase, project: Project, file: File, node: RedNode) -> TypeResult {
  if let Some(mapping) = YamlMapping::cast(node.clone()) {
    let is_top_level = mapping
      .syntax()
      .parent()
      .is_some_and(|parent| parent.kind() == SyntaxKind::YamlFrontmatter);
    if is_top_level {
      return get_mapping_type(db, project, file, &mapping);
    }
  }

  todo!();
}

fn get_mapping_type(
  db: &TypedownDatabase,
  project: Project,
  file: File,
  mapping: &YamlMapping,
) -> TypeResult {
  for (key, value_expr) in mapping.entries() {
    if key == "_type" {
      let resolved = referee(db, project, file, value_expr.syntax().clone());
      if let Some(symbol) = resolved.value(db) {
        return evaluate_schema(db, symbol);
      }

      return TypeResult::new(
        db,
        Box::new(TdrObjectType::get(db)),
        vec![Diagnostic::UnresolvedSchema {
          name: value_expr.syntax().text(),
          start_offset: value_expr.syntax().offset(),
          end_offset: value_expr.syntax().offset() + value_expr.syntax().text_len(),
        }],
      );
    }
  }

  TypeResult::new(
    db,
    Box::new(TdrObjectType::get(db)),
    vec![Diagnostic::MissingSchemaField {
      start_offset: mapping.syntax().offset(),
      end_offset: mapping.syntax().offset() + mapping.syntax().text_len(),
    }],
  )
}

#[cfg(test)]
mod tests {
  use std::{collections::HashMap, path::PathBuf};

  use typedown_syntax::ast::{AstNode, SourceFile};

  use crate::{
    QueryStorage, TypedownDatabase,
    derived::{
      get_builtin_types::get_schema_type, parse_file::parse_file, typechecker::get_type::get_type,
    },
    inputs::{File, FileHandle},
    types::{Project, TdrProductType, TdrTypeLike},
  };

  fn vault_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/evaluate_schema/my_vault")
  }

  #[test]
  fn get_type_of_schema_file_top_level_mapping_is_schema_type() {
    let vault = vault_root();
    let schema_file_path = vault.join("schemas/Person.tdr");

    let db = TypedownDatabase {
      storage: QueryStorage::default(),
    };

    let file = File::new(&db, FileHandle::Path(schema_file_path.clone()));
    let handles = HashMap::from([(schema_file_path, file.handle(&db))]);
    let project = Project::new(&db, vault, handles);

    let parse_result = parse_file(&db, project, file);
    let root = parse_result.ast(&db);

    let source_file = SourceFile::cast(root).expect("root should be SourceFile");
    let mapping = source_file
      .frontmatter()
      .expect("schema file should have frontmatter")
      .mapping()
      .expect("frontmatter should have a mapping");

    let type_result = get_type(&db, project, file, mapping.syntax().clone());

    let expected = Box::new(get_schema_type(&db)) as Box<dyn TdrTypeLike>;
    assert!(
      type_result.typ(&db) == expected,
      "top-level mapping of a schema file should have type TdrSchemaType"
    );
    assert!(
      type_result.diagnostics(&db).is_empty(),
      "expected no diagnostics, got: {:?}",
      type_result.diagnostics(&db)
    );
  }
}
