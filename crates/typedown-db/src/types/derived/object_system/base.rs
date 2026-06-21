use std::any::Any;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use super::func::TdrFuncObj;
use super::str::{TdrStrObj, TdrStrType};
use crate::derived::get_builtin_types::{get_object_type, get_str_type, get_type_type};
use crate::types::{FuncSignature, InstResult, MemberType, TypeMember, TypeMemberDescriptors};
use crate::{Id, TypedownDatabase};
use dyn_clone::{DynClone, clone_trait_object};
use typedown_macros::query_derived;

// Everything is an object
pub trait TdrObjectLike: Id + Any + DynClone + Send + Sync {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike>;

  fn lookup_method(&self, db: &TypedownDatabase, key: &str) -> Option<TdrFuncObj> {
    let mut current = self.get_type(db);
    loop {
      if let Some(func_obj) = current.get_vtable(db).remove(key) {
        return Some(func_obj);
      }
      let supertype = current.get_supertype(db);
      if supertype.as_id() == current.as_id() {
        return None;
      }
      current = supertype;
    }
  }

  fn lookup_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>> {
    if let Some(field) = self.get_owned_field(db, key) {
      return Some(field);
    }
    self
      .lookup_method(db, key)
      .map(|func_obj| Box::new(func_obj) as Box<dyn TdrObjectLike>)
  }

  fn get_owned_field(&self, db: &TypedownDatabase, key: &str) -> Option<Box<dyn TdrObjectLike>>;

  fn as_type(&self) -> Option<Box<dyn TdrTypeLike>> {
    None
  }
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
      MemberType::Simple(Box::new(TdrTypeType::get(db))),
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
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncObj>;
  fn get_owned_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember>;

  // This function doesn't handle arity cecking
  fn instantiate(&self, db: &TypedownDatabase, args: Vec<Box<dyn TdrTypeLike>>) -> InstResult;

  fn is_compatible_with(&self, db: &TypedownDatabase, actual: &dyn TdrTypeLike) -> bool;

  fn get_type_args(&self, db: &TypedownDatabase) -> Vec<Box<dyn TdrTypeLike>>;

  fn display_name(&self, db: &TypedownDatabase) -> String;

  fn construct(
    &self,
    db: &TypedownDatabase,
    args: Vec<Box<dyn TdrObjectLike>>,
  ) -> Option<Box<dyn TdrObjectLike>>;

  fn get_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<TypeMember> {
    if let Some(field) = get_builtin_field(db, name) {
      return Some(field);
    }
    if let Some(field) = self.get_owned_field_type(db, name) {
      return Some(field);
    }
    let supertype = self.get_supertype(db);
    // Stop when supertype is identical to self
    if supertype.as_id() == self.as_id() {
      return None;
    }
    supertype.get_field_type(db, name)
  }

  // Walk this type's own vtable for an instance method
  fn lookup_instance_method(&self, db: &TypedownDatabase, key: &str) -> Option<TdrFuncObj> {
    if let Some(func_obj) = self.get_vtable(db).remove(key) {
      return Some(func_obj);
    }
    let supertype = self.get_supertype(db);
    if supertype.as_id() == self.as_id() {
      return None;
    }
    supertype.lookup_instance_method(db, key)
  }

  // Look up a field or method type
  fn lookup_field_type(&self, db: &TypedownDatabase, name: &str) -> Option<Box<dyn TdrTypeLike>> {
    if let Some(member) = self.get_field_type(db, name) {
      if let MemberType::Simple(typ) = member.typ(db) {
        return Some(typ);
      }
    }
    self
      .lookup_instance_method(db, name)
      .map(|func_obj| func_obj.get_type(db))
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

/// The metatype is is the type of all types
/// It's an instance of itself & the type of every type.
#[query_derived]
pub struct TdrTypeType {}

impl TdrObjectLike for TdrTypeType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
  fn as_type(&self) -> Option<Box<dyn TdrTypeLike>> {
    Some(Box::new(self.clone()))
  }
}

impl TdrTypeLike for TdrTypeType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }
  fn get_supertype(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_vtable(&self, _db: &TypedownDatabase) -> HashMap<String, TdrFuncObj> {
    HashMap::new()
  }
  fn get_owned_field_type(&self, _db: &TypedownDatabase, _name: &str) -> Option<TypeMember> {
    None
  }
  fn instantiate(&self, db: &TypedownDatabase, _args: Vec<Box<dyn TdrTypeLike>>) -> InstResult {
    InstResult::new(db, Box::new(self.clone()), vec![])
  }

  fn get_type_args(&self, _db: &TypedownDatabase) -> Vec<Box<dyn TdrTypeLike>> {
    vec![]
  }

  fn is_compatible_with(&self, _db: &TypedownDatabase, actual: &dyn TdrTypeLike) -> bool {
    self.as_id() == actual.as_id()
  }

  fn construct(
    &self,
    _db: &TypedownDatabase,
    _args: Vec<Box<dyn TdrObjectLike>>,
  ) -> Option<Box<dyn TdrObjectLike>> {
    // HIR-level construction (ident/mapping paths) lives in utils.rs
    None
  }

  fn display_name(&self, _db: &TypedownDatabase) -> String {
    "type".to_string()
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
  fn as_type(&self) -> Option<Box<dyn TdrTypeLike>> {
    Some(Box::new(self.clone()))
  }
}

impl TdrTypeLike for TdrObjectType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }
  // Supertype of ObjectType is itself
  fn get_supertype(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncObj> {
    // Default to_string: returns the type's display name
    let sig = FuncSignature::new(db, vec![], Box::new(TdrStrType::get(db)));
    let func_obj = TdrFuncObj::new(
      db,
      "to_string".to_string(),
      Box::new(TdrObjectType::get(db)),
      sig,
      object_to_string,
    );
    HashMap::from([("to_string".to_string(), func_obj)])
  }
  fn get_owned_field_type(&self, _db: &TypedownDatabase, _name: &str) -> Option<TypeMember> {
    None
  }
  fn instantiate(&self, db: &TypedownDatabase, _args: Vec<Box<dyn TdrTypeLike>>) -> InstResult {
    InstResult::new(db, Box::new(self.clone()), vec![])
  }

  fn get_type_args(&self, _db: &TypedownDatabase) -> Vec<Box<dyn TdrTypeLike>> {
    vec![]
  }

  fn is_compatible_with(&self, _db: &TypedownDatabase, actual: &dyn TdrTypeLike) -> bool {
    self.as_id() == actual.as_id()
  }

  fn construct(
    &self,
    _db: &TypedownDatabase,
    args: Vec<Box<dyn TdrObjectLike>>,
  ) -> Option<Box<dyn TdrObjectLike>> {
    args.into_iter().next()
  }

  fn display_name(&self, _db: &TypedownDatabase) -> String {
    "object".to_string()
  }
}

impl TdrObjectType {
  pub fn get(db: &TypedownDatabase) -> TdrObjectType {
    get_object_type(db)
  }
}

fn object_to_string(
  db: &TypedownDatabase,
  this: Box<dyn TdrObjectLike>,
  _args: Vec<Box<dyn TdrObjectLike>>,
) -> Option<Box<dyn TdrObjectLike>> {
  let display = this.get_type(db).display_name(db);
  Some(Box::new(TdrStrObj::new(db, display)))
}
