use typedown_macros::query_derived;

use crate::derived::name_resolver::file_symbol::MaybeSymbol;
use crate::derived::name_resolver::members::members;
use crate::derived::name_resolver::scope::{parent_scope, scope};
use crate::types::{HirValue, HirValueKind};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn referee(db: &TypedownDatabase, hir: HirValue) -> MaybeSymbol {
  let name = match hir.kind(db) {
    HirValueKind::Ident(name) => name,
    _ => return MaybeSymbol::new(db, None),
  };

  let mut current_scope = scope(db, hir);
  loop {
    let result = members(db, current_scope);
    if let Some(sym) = result.members(db).get(&name) {
      return MaybeSymbol::new(db, Some(*sym));
    }
    match parent_scope(db, current_scope).value(db) {
      Some(parent) => current_scope = parent,
      None => return MaybeSymbol::new(db, None),
    }
  }
}
