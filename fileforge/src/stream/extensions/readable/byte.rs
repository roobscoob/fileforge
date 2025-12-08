use crate::{
  encoding::{Encoder, Encoding},
  stream::ReadableStream,
};

pub trait BinaryReadableStreamExt: ReadableStream<Type = u8> {
  fn decode<E: Encoding>(self) -> E::Encoder<Self>;
}

impl<S: ReadableStream<Type = u8>> BinaryReadableStreamExt for S {
  fn decode<E: Encoding>(self) -> E::Encoder<Self> {
    E::Encoder::new(self)
  }
}
