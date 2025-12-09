use core::convert::Infallible;

use fileforge::{binary_reader::BinaryReader, stream::ReadableStream};

use crate::byml::node::BymlConstructable;

pub struct BymlFloat32Node {
  value: f32,
}

impl<'pool, S: ReadableStream<Type = u8>, E> BymlConstructable<'pool, S, E> for BymlFloat32Node {
  type Error = Infallible;

  async fn construct<F: AsyncFnOnce(u64) -> Result<BinaryReader<'pool, S>, E>>(value: u32, _: F) -> Result<Self, Infallible> {
    Ok(Self {
      value: f32::from_ne_bytes(value.to_ne_bytes()),
    })
  }
}
