use std::collections::HashMap;

use super::func::TdrFuncType;
use crate::TypedownDatabase;
use crate::derived::get_builtin_types::get_object_type;
use typedown_macros::query_derived;

// Everything is an object
pub trait TdrObjectLike: Send + Sync {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike>;

  fn lookup_method(&self, db: &TypedownDatabase, key: &str) -> Option<TdrFuncType> {
    let typ = self.get_type(db);
    typ.get_vtable(db).remove(key)
  }

  fn lookup_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    self.get_owned_fields(db).remove(key)
  }

  fn get_owned_fields(&self, db: &TypedownDatabase) -> HashMap<String, Box<dyn TdrObjectLike>>;
}

impl<T: TdrObjectLike + ?Sized> TdrObjectLike for &T {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    (**self).get_type(db)
  }
  fn get_owned_fields(&self, db: &TypedownDatabase) -> HashMap<String, Box<dyn TdrObjectLike>> {
    (**self).get_owned_fields(db)
  }
}

impl<T: TdrObjectLike + ?Sized> TdrObjectLike for Box<T> {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    (**self).get_type(db)
  }
  fn get_owned_fields(&self, db: &TypedownDatabase) -> HashMap<String, Box<dyn TdrObjectLike>> {
    (**self).get_owned_fields(db)
  }
}

bitflags::bitflags! {
  #[derive(Clone, Copy, PartialEq, Eq)]
  pub struct TypeMemberDescriptors: u8 {
    const OPTIONAL = 0b0000_0001;
  }
}

pub struct TypeMember {
  pub typ: Box<dyn TdrTypeLike>,
  pub descriptors: TypeMemberDescriptors,
}

pub trait TdrTypeLike: TdrObjectLike {
  fn get_supertype(&self, db: &TypedownDatabase) -> Option<Box<dyn TdrTypeLike>>;
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncType>;
  fn get_owned_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember>;

  fn get_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
    self.get_owned_field_type(db, name)
      .or_else(|| self.get_supertype(db)?.get_field_type(db, name))
  }
}

/// The base type for all objects in TDR
#[query_derived]
pub struct TdrObjectType {}

impl TdrObjectLike for TdrObjectType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_owned_fields(&self, db: &TypedownDatabase) -> HashMap<String, Box<dyn TdrObjectLike>> {
    HashMap::new()
  }
}

impl TdrTypeLike for TdrObjectType {
  fn get_supertype(&self, db: &TypedownDatabase) -> Option<Box<dyn TdrTypeLike>> {
    None
  }
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncType> {
    HashMap::new()
  }
  fn get_owned_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
    todo!()
  }
}

impl TdrObjectType {
  pub fn get(db: &TypedownDatabase) -> TdrObjectType {
    get_object_type(db)
  }
}
