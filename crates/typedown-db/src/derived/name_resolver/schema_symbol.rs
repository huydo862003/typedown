use std::ops::Deref;

use typedown_macros::query_derived;
use typedown_syntax::ast::{AstNode, SourceFile, YamlFrontmatter, YamlMapping};

use crate::derived::get_vault_config::get_vault_config;
use crate::derived::name_resolver::symbol::symbol;
use crate::types::{File, GreenNode, Project, Symbol};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn schema_symbol(
  db: &TypedownDatabase,
  project: Project,
  file: File,
  node: GreenNode,
) -> Symbol {
  let mapping_node =
    resolve_mapping(db, node).expect("node is not a SourceFile, YamlFrontmatter, or YamlMapping");

  let config = get_vault_config(db, project);
  let schema_dir = config.schema_dir(db);
  let handles = project.handles(db);
  let file_handle = file.handle(db);

  let is_schema_file = handles
    .iter()
    .any(|(path, handle)| *handle == file_handle && path.starts_with(&schema_dir));

  assert!(is_schema_file, "file is not in the schema directory");

  symbol(db, project, file, mapping_node).expect("schema mapping should always produce a symbol")
}

fn resolve_mapping(db: &TypedownDatabase, node: GreenNode) -> Option<GreenNode> {
  if node.try_cast::<YamlMapping>(db).is_some() {
    return Some(node);
  }
  if let Some(frontmatter) = node.try_cast::<YamlFrontmatter>(db) {
    let mapping = frontmatter.mapping()?;
    return Some(GreenNode::new(db, mapping.syntax().deref().clone()));
  }
  if let Some(source_file) = node.try_cast::<SourceFile>(db) {
    let mapping = source_file.frontmatter()?.mapping()?;
    return Some(GreenNode::new(db, mapping.syntax().deref().clone()));
  }
  None
}
