//! Tracked query to get the resolved type of a HIR value.
//! The declared type takes precedence over the inferred type.

use tdr_macros::query_derived;

use crate::db::TypedownDatabase;
use crate::db::derived::typechecker::declared_node_type::declared_node_type;
use crate::db::derived::typechecker::infer_node_type::infer_node_type;
use crate::db::types::{HirValue, MemberType, TypeResult};
use tdr_incremental::QueryDatabase;

#[query_derived]
pub fn resolved_node_type(db: &TypedownDatabase, hir: HirValue) -> TypeResult {
  let declared = declared_node_type(db, hir);

  // If there is a declared member with a simple type, use it
  if let Some(member) = declared.member(db)
    && let MemberType::Simple(typ) = member.typ(db)
  {
    let diagnostics = declared.diagnostics(db).clone();
    return TypeResult::new(db, Some(typ), diagnostics);
  }

  // Fall back to inferred type
  infer_node_type(db, hir)
}
