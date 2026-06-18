//! Evaluate a resource file into typed objects

use typedown_macros::query_derived;
use typedown_syntax::ast::{AstNode, SourceFile};
use typedown_types::diagnostic::Diagnostic;

use crate::derived::hir::lower_expr;
use crate::derived::parse_file::parse_file;
use crate::derived::typechecker::get_node_type::get_node_type;
use crate::derived::typechecker::typecheck::typecheck;
use crate::types::{HirValue, ResourceResult, Symbol, SymbolKind, TdrObjectLike};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn evaluate_resource(db: &TypedownDatabase, symbol: Symbol) -> ResourceResult {
  let (project, file) = match symbol.kind(db) {
    SymbolKind::UserDefinedResource(project, file) => (project, file),
    _ => return ResourceResult::new(db, None, vec![]),
  };

  let mut diagnostics = vec![];

  // Parse file and lower frontmatter to HIR
  let parse_result = parse_file(db, project, file);
  diagnostics.extend(parse_result.diagnostics(db).iter().cloned());
  let root = parse_result.ast(db);
  let source_file = match SourceFile::cast(root) {
    Some(sf) => sf,
    None => return ResourceResult::new(db, None, diagnostics),
  };
  let mapping = match source_file.frontmatter().and_then(|fm| fm.mapping()) {
    Some(m) => m,
    None => return ResourceResult::new(db, None, diagnostics),
  };
  let hir = lower_expr(db, project, file, mapping.syntax().clone());

  // Typecheck the resource
  let typecheck_result = typecheck(db, hir);
  diagnostics.extend(typecheck_result.diagnostics(db).iter().cloned());

  // Construct the object using the type's constructor
  let obj = construct_from_hir(db, hir);

  ResourceResult::new(db, obj, diagnostics)
}

fn construct_from_hir(db: &TypedownDatabase, hir: HirValue) -> Option<Box<dyn TdrObjectLike>> {
  let type_result = get_node_type(db, hir);
  let typ = type_result.typ(db)?;
  typ.construct(db, hir)
}
