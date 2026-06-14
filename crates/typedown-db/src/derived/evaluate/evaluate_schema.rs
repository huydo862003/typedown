//! Evaluate a schema symbol to extract its type

use typedown_macros::query_derived;

use crate::types::{Symbol, TypeResult};
use crate::{QueryDatabase, TypedownDatabase};

#[query_derived]
pub fn evaluate_schema(db: &TypedownDatabase, symbol: Symbol) -> TypeResult {
  todo!();
}
