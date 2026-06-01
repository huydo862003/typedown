//! A salsa database for incremental Typedown compilation
//! Salsa seems to remove the ParallelDatabase trait: https://github.com/salsa-rs/salsa/pull/1013
//! I've scoured the salsa repo but it seems we have to do it manually now
pub mod inputs;
pub mod tracked;

#[salsa::db]
pub struct TypedownDatabase {
  storage: salsa::Storage<Self>,
}

#[salsa::db]
impl salsa::Database for TypedownDatabase {}
