use typedown_macros::query_derived;
use typedown_syntax::ast::YamlMapping;

use crate::derived::get_vault_config::get_vault_config;
use crate::types::{File, TdrNode, Project, Symbol, SymbolKind};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn symbol(
  db: &TypedownDatabase,
  project: Project,
  file: File,
  node: TdrNode,
) -> Option<Symbol> {
  if node.try_cast::<YamlMapping>(db).is_some() {
    let config = get_vault_config(db, project);
    let schema_dir = config.schema_dir(db);
    let handles = project.handles(db);
    let file_handle = file.handle(db);

    let is_schema_file = handles
      .iter()
      .any(|(path, handle)| *handle == file_handle && path.starts_with(&schema_dir));

    if is_schema_file {
      return Some(Symbol::new(db, node, SymbolKind::Schema));
    }
  }

  todo!()
}
