use std::any::Any;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use super::func::TdrFuncType;
use crate::derived::get_builtin_types::{
  get_object_type, get_schema_type, get_str_type, get_type_type,
};
use crate::types::{MemberType, TypeMember, TypeMemberDescriptors};
use crate::{Id, TypedownDatabase};
use dyn_clone::{DynClone, clone_trait_object};
use typedown_macros::query_derived;

// Everything is an object
pub trait TdrObjectLike: Id + Any + DynClone + Send + Sync {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike>;

  fn lookup_method(&self, db: &TypedownDatabase, key: &str) -> Option<TdrFuncType> {
    let typ = self.get_type(db);
    typ.get_vtable(db).remove(key)
  }

  fn lookup_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    self.get_owned_field(db, key)
  }

  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>>;
}

clone_trait_object!(TdrObjectLike);

impl PartialEq for Box<dyn TdrObjectLike> {
  fn eq(&self, other: &Self) -> bool {
    (**self).as_id() == (**other).as_id()
  }
}
impl Eq for Box<dyn TdrObjectLike> {}

impl Hash for Box<dyn TdrObjectLike> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    (**self).as_id().hash(state);
  }
}

fn get_builtin_field(db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
  match name {
    "_type" => Some(TypeMember::new(
      db,
      MemberType::Simple(Box::new(get_schema_type(db))),
      TypeMemberDescriptors::empty(),
    )),
    "_label" => Some(TypeMember::new(
      db,
      MemberType::Simple(Box::new(get_str_type(db))),
      TypeMemberDescriptors::OPTIONAL,
    )),
    _ => None,
  }
}

pub trait TdrTypeLike: TdrObjectLike + DynClone {
  fn arity(&self, db: &TypedownDatabase) -> usize;
  fn get_supertype(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike>;
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncType>;
  fn get_owned_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember>;

  // This function doesn't handle arity cecking
  fn instantiate(
    &self,
    db: &TypedownDatabase,
    args: Vec<Box<dyn TdrTypeLike>>,
  ) -> Box<dyn TdrTypeLike>;

  fn is_compatible_with(&self, db: &TypedownDatabase, actual: &dyn TdrTypeLike) -> bool;

  fn get_type_args(&self, db: &TypedownDatabase) -> Vec<Box<dyn TdrTypeLike>>;

  fn get_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
    if let Some(field) = get_builtin_field(db, name) {
      return Some(field);
    }
    if let Some(field) = self.get_owned_field_type(db, name) {
      return Some(field);
    }
    let supertype = self.get_supertype(db);
    // Stop when supertype is identical to self (e.g. TdrObjectType, which is its own supertype).
    if supertype.as_id() == self.as_id() {
      return None;
    }
    supertype.get_field_type(db, name)
  }
}

clone_trait_object!(TdrTypeLike);

impl PartialEq for Box<dyn TdrTypeLike> {
  fn eq(&self, other: &Self) -> bool {
    (**self).as_id() == (**other).as_id()
  }
}
impl Eq for Box<dyn TdrTypeLike> {}

impl Hash for Box<dyn TdrTypeLike> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    (**self).as_id().hash(state);
  }
}

/// The top type: an instance of itself, supertype of everything.
#[query_derived]
pub struct TdrTypeType {}

impl TdrObjectLike for TdrTypeType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

impl TdrTypeLike for TdrTypeType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }
  fn get_supertype(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_vtable(&self, _db: &TypedownDatabase) -> HashMap<String, TdrFuncType> {
    HashMap::new()
  }
  fn get_owned_field_type(&self, _db: &TypedownDatabase, _name: &str) -> Option<TypeMember> {
    None
  }
  fn instantiate(
    &self,
    _db: &TypedownDatabase,
    _args: Vec<Box<dyn TdrTypeLike>>,
  ) -> Box<dyn TdrTypeLike> {
    Box::new(self.clone())
  }

  fn get_type_args(&self, _db: &TypedownDatabase) -> Vec<Box<dyn TdrTypeLike>> {
    vec![]
  }

  fn is_compatible_with(&self, _db: &TypedownDatabase, actual: &dyn TdrTypeLike) -> bool {
    self.as_id() == actual.as_id()
  }
}

impl TdrTypeType {
  pub fn get(db: &TypedownDatabase) -> TdrTypeType {
    get_type_type(db)
  }
}

/// The base type for all objects in TDR
#[query_derived]
pub struct TdrObjectType {}

impl TdrObjectLike for TdrObjectType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
}

impl TdrTypeLike for TdrObjectType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }
  // Supertype of ObjectType is itself: this is the termination point.
  fn get_supertype(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_vtable(&self, _db: &TypedownDatabase) -> HashMap<String, TdrFuncType> {
    HashMap::new()
  }
  fn get_owned_field_type(&self, _db: &TypedownDatabase, _name: &str) -> Option<TypeMember> {
    None
  }
  fn instantiate(
    &self,
    _db: &TypedownDatabase,
    _args: Vec<Box<dyn TdrTypeLike>>,
  ) -> Box<dyn TdrTypeLike> {
    Box::new(self.clone())
  }

  fn get_type_args(&self, _db: &TypedownDatabase) -> Vec<Box<dyn TdrTypeLike>> {
    vec![]
  }

  fn is_compatible_with(&self, _db: &TypedownDatabase, actual: &dyn TdrTypeLike) -> bool {
    self.as_id() == actual.as_id()
  }
}

impl TdrObjectType {
  pub fn get(db: &TypedownDatabase) -> TdrObjectType {
    get_object_type(db)
  }
}
