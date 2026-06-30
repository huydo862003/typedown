use typedown_macros::query_derived;

use crate::derived::get_vault_config::get_vault_config;
use crate::types::{File, FileHandle, Project, Symbol, SymbolKind};
use crate::{
  Decodable, Decoder, Encodable, Encoder, QueryDatabase, StableHash, StableHasher, TypedownDatabase,
};

#[query_derived]
pub struct MaybeSymbol {
  pub value: Option<Symbol>,
}

impl StableHash<TypedownDatabase> for MaybeSymbol {
  fn stable_hash(&self, db: &TypedownDatabase, hasher: &mut StableHasher) {
    self.value(db).stable_hash(db, hasher);
  }
}

impl Encodable<TypedownDatabase> for MaybeSymbol {
  fn encode(&self, encoder: &mut Encoder<TypedownDatabase>) {
    self.value(encoder.db).encode(encoder);
  }
}

impl Decodable<TypedownDatabase> for MaybeSymbol {
  fn decode(decoder: &mut Decoder<TypedownDatabase>) -> Self {
    let value = Option::decode(decoder);
    MaybeSymbol::new(decoder.db, value)
  }
}

#[query_derived]
pub fn file_symbol(db: &TypedownDatabase, project: Project, file: File) -> MaybeSymbol {
  let config = get_vault_config(db, project);
  let schema_dir = config.schema_dir(db);
  let proj_files = project.files(db);

  let is_schema_file = proj_files
    .iter()
    .any(|(path, proj_file)| *proj_file == file && path.starts_with(&schema_dir));

  let name = match file.handle(db) {
    FileHandle::Path(path, _) => path
      .file_stem()
      .and_then(|s| s.to_str())
      .unwrap_or_default()
      .to_string(),
    FileHandle::Content(_) => String::new(),
  };

  let kind = if is_schema_file {
    SymbolKind::UserDefinedSchema(project, file)
  } else {
    SymbolKind::UserDefinedResource(project, file)
  };

  MaybeSymbol::new(db, Some(Symbol::new(db, kind, name)))
}
