use core::convert::Infallible;

use fileforge::{binary_reader::BinaryReader, stream::ReadableStream};

use crate::byml::node::BymlConstructable;

pub struct BymlStringNode {
  string_table_index: u32,
}

impl<'pool, S: ReadableStream<Type = u8>, E> BymlConstructable<'pool, S, E> for BymlStringNode {
  type Error = Infallible;

  async fn construct<F: AsyncFnOnce(u64) -> Result<BinaryReader<'pool, S>, E>>(value: u32, _: F) -> Result<Result<Self, Self::Error>, E> {
    Ok(Ok(Self { string_table_index: value }))
  }
}
