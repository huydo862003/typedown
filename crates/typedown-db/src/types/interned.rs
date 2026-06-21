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

/// The type of a type member field
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum MemberType {
  /// A reference to a named type (e.g. `string`, `list[number]`)
  Simple(Box<dyn TdrTypeLike>),
  /// A union or enum type: each arm is itself a `TypeMember` (a type ref)
  Sum(Vec<TypeMember>),
  /// A literal value constraint (e.g. `"foo"`, `42`, `true`)
  Literal(LiteralValue),
  /// The bottom type: no value can be assigned to this field
  Never,
}

/// A concrete literal value used in literal constraints
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LiteralValue {
  Str(String),
  Bool(bool),
  // f64 cannot be hashed so we store in string
  Num(String),
}

#[query_interned]
pub struct TypeMember {
  pub typ: MemberType,
  pub descriptors: TypeMemberDescriptors,
}

