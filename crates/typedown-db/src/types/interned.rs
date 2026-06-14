use typedown_macros::query_interned;

use super::TdrTypeLike;

#[query_interned]
pub struct FuncSignature {
  pub params: Vec<Box<dyn TdrTypeLike>>,
  pub ret: Box<dyn TdrTypeLike>,
}

bitflags::bitflags! {
  #[derive(Clone, Copy, PartialEq, Eq, Hash)]
  pub struct TypeMemberDescriptors: u8 {
    const OPTIONAL = 0b0000_0001;
  }
}

#[query_interned]
pub struct TypeMember {
  pub typ: Box<dyn TdrTypeLike>,
  pub descriptors: TypeMemberDescriptors,
}
