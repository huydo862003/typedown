use typedown_macros::query_interned;

use super::TdrTypeLike;

#[query_interned]
pub struct FuncSignature {
  pub params: Vec<Box<dyn TdrTypeLike>>,
  pub ret: Box<dyn TdrTypeLike>,
}
