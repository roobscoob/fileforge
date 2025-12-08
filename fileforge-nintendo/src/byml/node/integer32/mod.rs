use core::convert::Infallible;

use fileforge::{binary_reader::BinaryReader, stream::ReadableStream};

use crate::byml::node::BymlConstructable;

pub struct BymlInteger32Node {
  value: i32,
}

impl<'pool, S: ReadableStream<Type = u8>, E> BymlConstructable<'pool, S, E> for BymlInteger32Node {
  type Error = Infallible;

  async fn construct<F: AsyncFnOnce(u64) -> Result<BinaryReader<'pool, S>, E>>(value: u32, _: F) -> Result<Result<Self, Self::Error>, E> {
    Ok(Ok(Self {
      value: i32::from_ne_bytes(value.to_ne_bytes()),
    }))
  }
}
