use typedown_macros::query_derived;

use crate::types::{HirValue, Scope, ScopeKind};
use crate::{
  Decodable, Decoder, Encodable, Encoder, QueryDatabase, StableHash, StableHasher, TypedownDatabase,
};

#[query_derived]
pub struct MaybeScope {
  pub value: Option<Scope>,
}

impl StableHash<TypedownDatabase> for MaybeScope {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.value(db).stable_hash(db, hasher);
  }
}

impl Encodable<TypedownDatabase> for MaybeScope {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    self.value(encoder.db).encode(encoder);
  }
}

impl Decodable<TypedownDatabase> for MaybeScope {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    let value = Option::decode(decoder);
    MaybeScope::new(decoder.db, value)
  }
}

#[query_derived]
pub fn scope(db: &TypedownDatabase, hir: HirValue) -> Scope {
  Scope::file_scope(db, hir.project(db), hir.file(db))
}

#[query_derived]
pub fn parent_scope(db: &TypedownDatabase, scope: Scope) -> MaybeScope {
  match scope.kind(db) {
    ScopeKind::Builtin => MaybeScope::new(db, None),
    ScopeKind::Project(_) => MaybeScope::new(db, Some(Scope::builtin_scope(db))),
    ScopeKind::File(project, _) => MaybeScope::new(db, Some(Scope::project_scope(db, project))),
  }
}
