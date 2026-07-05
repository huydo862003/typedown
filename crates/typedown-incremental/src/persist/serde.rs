use crate::{Decoder, Encoder};

pub trait SerializeContext {
  fn encoder(&mut self) -> &mut dyn Encoder;
}

pub trait DeserializeContext {
  fn decoder(&mut self) -> &mut dyn Decoder;
}
