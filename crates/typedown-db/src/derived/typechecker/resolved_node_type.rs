//! Tracked query to get the resolved type of a HIR value.
//!
//! Reconciles the declared (top-down, schema-driven) type with the inferred
//! (bottom-up, value-driven) type. The declared type takes precedence when
//! present; otherwise the inferred type is used.

use typedown_macros::query_derived;

use crate::derived::typechecker::declared_node_type::declared_node_type;
use crate::derived::typechecker::infer_node_type::infer_node_type;
use crate::types::{HirValue, MemberType, TypeResult};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn resolved_node_type(db: &TypedownDatabase, hir: HirValue) -> TypeResult {
  let declared = declared_node_type(db, hir);

  // If there is a declared member with a simple type, use it
  if let Some(member) = declared.member(db) {
    if let MemberType::Simple(typ) = member.typ(db) {
      let diagnostics = declared.diagnostics(db).clone();
      return TypeResult::new(db, Some(typ), diagnostics);
    }
  }

  // Fall back to inferred type
  infer_node_type(db, hir)
}
