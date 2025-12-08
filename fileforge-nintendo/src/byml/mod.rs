use fileforge::{binary_reader::BinaryReader, stream::ReadableStream};

use crate::byml::{
  header::BymlHeader,
  node::{BymlAnyConstructable, BymlConstructionError, BymlNode, BymlNodeDiscriminants},
};

pub mod header;
pub mod node;

pub struct Byml<'pool, S: ReadableStream<Type = u8>> {
  header: BymlHeader,
  stream: BinaryReader<'pool, S>,
}

enum IntoDataAtError {}

impl<'pool, S: ReadableStream<Type = u8>> Byml<'pool, S> {
  async fn into_data_at(self, position: u64) -> Result<BinaryReader<'pool, S>, IntoDataAtError> {}

  async fn into_node(self, discriminant: BymlNodeDiscriminants, value: u32) -> Result<BymlNode<'pool, S>, BymlConstructionError<'pool, IntoDataAtError, S>> {
    BymlNode::construct(discriminant, value, async move |offset| self.into_data_at(offset).await).await
  }
}
