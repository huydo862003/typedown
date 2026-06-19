//! Tracked query to get the type of a symbol

use typedown_macros::query_derived;
use typedown_syntax::ast::{AstNode, SourceFile};

use crate::derived::hir::lower_expr;
use crate::derived::parse_file::parse_file;
use crate::derived::typechecker::get_node_type::get_node_type;
use crate::types::{Symbol, SymbolKind, TdrTypeType, TypeResult};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn get_symbol_type(db: &TypedownDatabase, symbol: Symbol) -> TypeResult {
  match symbol.kind(db) {
    // Schema symbols are types
    SymbolKind::BuiltinSchema(_) | SymbolKind::UserDefinedSchema(_, _) => {
      TypeResult::new(db, Some(Box::new(TdrTypeType::get(db))), vec![])
    }
    // Resource symbols get their type from their frontmatter
    SymbolKind::UserDefinedResource(project, file) => {
      let parse_result = parse_file(db, project, file);
      let root = parse_result.ast(db);
      let source_file = match SourceFile::cast(root) {
        Some(sf) => sf,
        None => return TypeResult::new(db, None, vec![]),
      };
      let mapping = match source_file.frontmatter().and_then(|fm| fm.mapping()) {
        Some(m) => m,
        None => return TypeResult::new(db, None, vec![]),
      };
      let hir = lower_expr(db, project, file, mapping.syntax().clone());
      get_node_type(db, hir)
    }
    // Macros don't have a type themselves
    SymbolKind::BuiltinMacro(_) => TypeResult::new(db, None, vec![]),
  }
}
