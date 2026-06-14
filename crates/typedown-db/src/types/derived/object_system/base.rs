use std::any::Any;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use super::func::TdrFuncType;
use crate::derived::get_builtin_types::{get_object_type, get_schema_type, get_str_type};
use crate::types::{TypeMember, TypeMemberDescriptors};
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
    (**self).type_id() == (**other).type_id() && (**self).as_id() == (**other).as_id()
  }
}
impl Eq for Box<dyn TdrObjectLike> {}

impl Hash for Box<dyn TdrObjectLike> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    (**self).type_id().hash(state);
    (**self).as_id().hash(state);
  }
}

fn get_builtin_type_members(db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
  match name {
    "_type" => Some(TypeMember::new(
      db,
      Box::new(get_schema_type(db)),
      TypeMemberDescriptors::empty(),
    )),
    "_label" => Some(TypeMember::new(
      db,
      Box::new(get_str_type(db)),
      TypeMemberDescriptors::empty(),
    )),
    _ => None,
  }
}

pub trait TdrTypeLike: TdrObjectLike + DynClone {
  fn get_supertype(&self, db: &TypedownDatabase) -> Option<Box<dyn TdrTypeLike>>;
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncType>;
  fn get_owned_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember>;

  fn get_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
    get_builtin_type_members(db, name)
      .or_else(|| self.get_owned_field_type(db, name))
      .or_else(|| self.get_supertype(db)?.get_field_type(db, name))
  }
}

clone_trait_object!(TdrTypeLike);

impl PartialEq for Box<dyn TdrTypeLike> {
  fn eq(&self, other: &Self) -> bool {
    (**self).type_id() == (**other).type_id() && (**self).as_id() == (**other).as_id()
  }
}
impl Eq for Box<dyn TdrTypeLike> {}

impl Hash for Box<dyn TdrTypeLike> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    (**self).type_id().hash(state);
    (**self).as_id().hash(state);
  }
}

/// The base type for all objects in TDR
#[query_derived]
pub struct TdrObjectType {}

impl TdrObjectLike for TdrObjectType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
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
    None
  }
}

impl TdrObjectType {
  pub fn get(db: &TypedownDatabase) -> TdrObjectType {
    get_object_type(db)
  }
}
