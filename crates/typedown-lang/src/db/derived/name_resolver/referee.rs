use typedown_macros::query_derived;

use crate::red::RedNode;
use typedown_types::syntax_kind::SyntaxKind;

use crate::db::TypedownDatabase;
use crate::db::derived::name_resolver::file_symbol::MaybeSymbol;
use crate::db::derived::name_resolver::members::members;
use crate::db::derived::name_resolver::scope::{parent_scope, scope};
use crate::db::types::{HirValue, HirValueKind};
use typedown_incremental::QueryDatabase;

#[query_derived]
pub fn referee(db: &TypedownDatabase, hir: HirValue) -> MaybeSymbol {
  let name = match hir.kind(db) {
    HirValueKind::Ident(name) => name,
    _ => return MaybeSymbol::new(db, None),
  };

  if is_dot_rhs(&hir.node(db)) {
    return MaybeSymbol::new(db, None);
  }

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

// Returns true if `node` is the right-hand operand of a dot binary expression.
fn is_dot_rhs(node: &RedNode) -> bool {
  let parent = match node.parent() {
    Some(parent) => parent,
    None => return false,
  };
  if parent.kind() != SyntaxKind::BinaryExpr {
    return false;
  }
  let dot_op = parent
    .children()
    .find(|child| child.kind() == SyntaxKind::YamlOp && child.text() == ".");
  match dot_op {
    Some(op) => node.offset() > op.offset(),
    None => false,
  }
}
