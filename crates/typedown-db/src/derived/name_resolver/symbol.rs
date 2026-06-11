use std::ops::Deref;

use typedown_macros::query_derived;
use typedown_syntax::ast::{AstNode, SourceFile, YamlFrontmatter, YamlMapping};

use crate::derived::get_vault_config::get_vault_config;
use crate::types::{File, GreenNode, Project, Symbol, SymbolKind};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn symbol(db: &TypedownDatabase, project: Project, file: File, node: GreenNode) -> Symbol {
  if let Some(mapping) = try_as_schema_mapping(db, project, file, node) {
    return Symbol::new(db, mapping, SymbolKind::Schema);
  }

  todo!()
}

/// Given a node that could be a SourceFile, YamlFrontmatter, or YamlMapping in a schema file,
/// return the interned GreenNode for the top-level YamlMapping if applicable.
fn try_as_schema_mapping(
  db: &TypedownDatabase,
  project: Project,
  file: File,
  node: GreenNode,
) -> Option<GreenNode> {
  // Navigate down to the YamlMapping regardless of which level we were given
  let mapping_green = if node.try_cast::<YamlMapping>(db).is_some() {
    node
  } else if let Some(frontmatter) = node.try_cast::<YamlFrontmatter>(db) {
    let mapping = frontmatter.mapping()?;
    GreenNode::new(db, mapping.syntax().deref().clone())
  } else if let Some(source_file) = node.try_cast::<SourceFile>(db) {
    let frontmatter = source_file.frontmatter()?;
    let mapping = frontmatter.mapping()?;
    GreenNode::new(db, mapping.syntax().deref().clone())
  } else {
    return None;
  };

  // Check that the file is in the schema directory
  let config = get_vault_config(db, project);
  let schema_dir = config.schema_dir(db);
  let handles = project.handles(db);
  let file_handle = file.handle(db);

  let is_schema_file = handles
    .iter()
    .any(|(path, handle)| *handle == file_handle && path.starts_with(&schema_dir));

  if is_schema_file {
    Some(mapping_green)
  } else {
    None
  }
}
