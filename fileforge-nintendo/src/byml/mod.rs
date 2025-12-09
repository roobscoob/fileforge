use fileforge::{
  binary_reader::{error::SkipError, BinaryReader},
  stream::ReadableStream,
};
use fileforge_macros::FileforgeError;

use crate::byml::{
  header::BymlHeader,
  node::{discriminant::BymlNodeDiscriminantVersionConfig, BymlAnyConstructable, BymlConstructionError, BymlNode, BymlNodeDiscriminants},
};

pub mod header;
pub mod node;
pub mod readable;

pub struct Byml<'pool, S: ReadableStream<Type = u8>> {
  header: BymlHeader,
  reader: BinaryReader<'pool, S>,
}

#[derive(FileforgeError)]
pub enum IntoDataAtError<'pool, S: ReadableStream> {
  #[report(&"Out Of Bounds")]
  OutOfBounds,
  SeekError(#[from] SkipError<'pool, S::SkipError>),
}

impl<'pool, S: ReadableStream<Type = u8>> Byml<'pool, S> {
  fn version(&self) -> BymlNodeDiscriminantVersionConfig {
    BymlNodeDiscriminantVersionConfig {
      version_number: self.header.version(),
      feat_binary_data_table: self.header.config().feat_binary_data_table,
    }
  }

  async fn into_data_at(mut self, position: u64) -> Result<BinaryReader<'pool, S>, IntoDataAtError<'pool, S>> {
    let offset = position.checked_sub(self.header.size()).ok_or(IntoDataAtError::OutOfBounds)?;
    self.reader.skip(offset).await?;

    Ok(self.reader)
  }

  async fn into_node(self, discriminant: BymlNodeDiscriminants, value: u32) -> Result<BymlNode<'pool, S>, BymlConstructionError<'pool, IntoDataAtError<'pool, S>, S>> {
    BymlNode::construct(discriminant, value, self.version(), async move |offset| self.into_data_at(offset).await).await
  }

  async fn into_node_dyn(self, value: u32) -> Result<BymlNode<'pool, S>, BymlConstructionError<'pool, IntoDataAtError<'pool, S>, S>> {
    BymlNode::construct_dyn(value, self.version(), async move |offset| self.into_data_at(offset).await).await
  }

  pub async fn into_literal_table(self) -> Result<Option<BymlNode<'pool, S>>, BymlConstructionError<'pool, IntoDataAtError<'pool, S>, S>> {
    match self.header.string_table_offset() {
      Some(v) => Ok(Some(self.into_node_dyn(v.get()).await?)),
      None => Ok(None),
    }
  }

  pub async fn into_key_table(self) -> Result<Option<BymlNode<'pool, S>>, BymlConstructionError<'pool, IntoDataAtError<'pool, S>, S>> {
    match self.header.key_table_offset() {
      Some(v) => Ok(Some(self.into_node_dyn(v.get()).await?)),
      None => Ok(None),
    }
  }
}
