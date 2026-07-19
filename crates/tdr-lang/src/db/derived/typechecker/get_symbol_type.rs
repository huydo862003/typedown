//! Tracked query to get the type of a symbol

use tdr_macros::query_derived;

use crate::db::TypedownDatabase;
use crate::db::derived::typechecker::actual_node_type::actual_node_type;
use crate::db::types::{
  MemberType, Symbol, SymbolKind, TdrTypeType, TypeMember, TypeMemberDescriptors, TypeMemberResult,
};
use crate::db::utils::lower_file;
use tdr_incremental::QueryDatabase;

#[query_derived]
pub fn get_symbol_type(db: &TypedownDatabase, symbol: Symbol) -> TypeMemberResult {
  match symbol.kind(db) {
    SymbolKind::BuiltinSchema(_) | SymbolKind::UserDefinedSchema(_, _) => TypeMemberResult::new(
      db,
      Some(TypeMember::new(
        db,
        MemberType::Simple(TdrTypeType::get(db).into()),
        TypeMemberDescriptors::empty(),
      )),
      vec![],
    ),
    SymbolKind::UserDefinedResource(project, file) => {
      let (hir, _) = lower_file(db, project, file);
      match hir {
        Some(hir) => actual_node_type(db, hir),
        None => TypeMemberResult::new(db, None, vec![]),
      }
    }
    SymbolKind::BuiltinMacro(_) => TypeMemberResult::new(db, None, vec![]),
  }
}
