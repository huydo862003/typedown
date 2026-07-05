use typedown_incremental::{Decoder, DeserializeContext, Encoder, SerializeContext};

use crate::db::{TypedownDecoder, TypedownEncoder};

/// Serialize context for typedown
pub struct TypedownSerializeContext<'a> {
  encoder: TypedownEncoder<'a>,
}

impl<'a> TypedownSerializeContext<'a> {}

impl<'a> SerializeContext for TypedownSerializeContext<'a> {
  fn encoder(&mut self) -> &mut dyn Encoder {
    &mut self.encoder
  }
}

/// Deserialize context for typedown
pub struct TypedownDeserializeContext<'a> {
  decoder: TypedownDecoder<'a>,
}

impl<'a> TypedownDeserializeContext<'a> {}

impl<'a> DeserializeContext for TypedownDeserializeContext<'a> {
  fn decoder(&mut self) -> &mut dyn Decoder {
    &mut self.decoder
  }
}
