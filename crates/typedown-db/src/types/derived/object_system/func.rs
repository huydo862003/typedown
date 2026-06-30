use std::collections::HashMap;
use typedown_macros::query_derived;

use super::base::{TdrObjectLike, TdrObjectType, TdrTypeLike, TdrTypeType};
use super::native_fn::NativeFnKind;
use super::str::TdrStrType;
use crate::derived::get_builtin_types::get_func_type;
use crate::types::{FuncSignature, InstResult, TypeMember};
use crate::{
  Decodable, Decoder, Encodable, Encoder, Id, StableHash, StableHasher, TypedownDatabase,
};

#[query_derived]
pub struct TdrFuncType {
  #[id]
  pub signature: FuncSignature,
}

impl TdrObjectLike for TdrFuncType {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrTypeType::get(db))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
  fn source_path(&self, db: &TypedownDatabase) -> String {
    let sig = self.signature(db);
    let params: Vec<String> = sig
      .params(db)
      .iter()
      .map(|param| param.source_path(db))
      .collect();
    let ret = sig.ret(db).source_path(db);
    format!("@builtin::function[({}) -> {}]", params.join(", "), ret)
  }

  fn as_type(&self) -> Option<Box<dyn TdrTypeLike>> {
    Some(Box::new(self.clone()))
  }
}

impl TdrTypeLike for TdrFuncType {
  fn arity(&self, _db: &TypedownDatabase) -> usize {
    0
  }

  fn get_supertype(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(TdrObjectType::get(db))
  }
  fn get_vtable(&self, db: &TypedownDatabase) -> HashMap<String, TdrFuncObj> {
    let sig = FuncSignature::new(db, vec![], Box::new(TdrStrType::get(db)));
    let func_obj = TdrFuncObj::new(
      db,
      "to_string".to_string(),
      Box::new(self.clone()),
      sig,
      NativeFnKind::FuncToString,
    );
    HashMap::from([("to_string".to_string(), func_obj)])
  }
  fn get_owned_field_type(&self, _db: &TypedownDatabase, _name: &str) -> Option<TypeMember> {
    None
  }
  fn instantiate(&self, db: &TypedownDatabase, args: Vec<Box<dyn TdrTypeLike>>) -> InstResult {
    assert_eq!(args.len(), self.arity(db), "arity mismatch");
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
    None
  }

  fn display_name(&self, _db: &TypedownDatabase) -> String {
    "function".to_string()
  }
}

impl TdrFuncType {
  pub fn get(
    db: &TypedownDatabase,
    params: Vec<Box<dyn TdrTypeLike>>,
    ret: Box<dyn TdrTypeLike>,
  ) -> TdrFuncType {
    get_func_type(db, FuncSignature::new(db, params, ret))
  }
}

impl StableHash<TypedownDatabase> for TdrFuncType {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.source_path(db).stable_hash(db, hasher);
    self.signature(db).stable_hash(db, hasher);
  }
}

impl Encodable<TypedownDatabase> for TdrFuncType {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    self.signature(encoder.db).encode(encoder);
  }
}

impl Decodable<TypedownDatabase> for TdrFuncType {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    let signature = FuncSignature::decode(decoder);
    TdrFuncType::new(decoder.db, signature)
  }
}

#[query_derived]
pub struct TdrFuncObj {
  #[id]
  pub name: String,
  #[id]
  pub typ: Box<dyn TdrTypeLike>,
  #[id]
  pub signature: FuncSignature,
  pub func: NativeFnKind,
}

impl TdrFuncObj {
  pub fn call(
    &self,
    db: &TypedownDatabase,
    this: Box<dyn TdrObjectLike>,
    args: Vec<Box<dyn TdrObjectLike>>,
  ) -> Option<Box<dyn TdrObjectLike>> {
    (self.func(db).resolve())(db, this, args)
  }
}

impl TdrObjectLike for TdrFuncObj {
  fn get_type(&self, db: &TypedownDatabase) -> Box<dyn TdrTypeLike> {
    Box::new(get_func_type(db, self.signature(db)))
  }
  fn get_owned_field(&self, _db: &TypedownDatabase, _key: &str) -> Option<Box<dyn TdrObjectLike>> {
    None
  }
  fn source_path(&self, db: &TypedownDatabase) -> String {
    self.get_type(db).source_path(db)
  }
}

impl StableHash<TypedownDatabase> for TdrFuncObj {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.name(db).stable_hash(db, hasher);
    self.typ(db).stable_hash(db, hasher);
    self.signature(db).stable_hash(db, hasher);
    self.func(db).stable_hash(db, hasher);
  }
}

impl Encodable<TypedownDatabase> for TdrFuncObj {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    self.name(encoder.db).encode(encoder);
    self.typ(encoder.db).encode(encoder);
    self.signature(encoder.db).encode(encoder);
    self.func(encoder.db).encode(encoder);
  }
}

impl Decodable<TypedownDatabase> for TdrFuncObj {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    let name = String::decode(decoder);
    let typ = Box::<dyn TdrTypeLike>::decode(decoder);
    let signature = FuncSignature::decode(decoder);
    let func = NativeFnKind::decode(decoder);
    TdrFuncObj::new(decoder.db, name, typ, signature, func)
  }
}
