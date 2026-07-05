//! Evaluate a HIR node into a typed object

use typedown_macros::query_derived;

use crate::TypedownDatabase;
use crate::derived::evaluate::utils::construct_from_hir;
use crate::derived::typechecker::typecheck::typecheck;
use crate::types::{HirValue, ResourceResult};
use typedown_incremental::QueryDatabase;

#[query_derived]
pub fn evaluate_node(db: &TypedownDatabase, hir: HirValue) -> ResourceResult {
  let mut diagnostics = vec![];

  let typecheck_result = typecheck(db, hir);
  diagnostics.extend(typecheck_result.diagnostics(db).iter().cloned());

  let obj = construct_from_hir(db, hir, &mut diagnostics);

  ResourceResult::new(db, obj, diagnostics)
}
