use fileforge::binary_reader::PrimitiveReader;
use fileforge::stream::builtin::read_until::ReadUntil;
use fileforge::stream::extensions::readable::ReadableStreamExt;
use fileforge::{
  binary_reader::{
    error::{common::Read, primitive_name_annotation::PrimitiveName, GetPrimitiveError, SkipError},
    readable::builtins::{
      array::ArrayReadError,
      contiugous::{Contiguous, ContiguousSkipError},
    },
    BinaryReader,
  },
  error::ext::annotations::annotated::Annotated,
  stream::{ReadableStream, StreamReadError, StreamSkipError, SINGLE},
  ResultIgnoreExt,
};
use fileforge_macros::FileforgeError;

use crate::byml::node::BymlConstructable;

pub struct BymlStringTableNode<'pool, S: ReadableStream<Type = u8>> {
  count: u32,
  reader: BinaryReader<'pool, S>,
}

pub enum BymlStringTableNodeConstructableError<'pool, S: ReadableStream<Type = u8>> {
  AcquireReaderError(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, S::ReadError>>),
}

impl<'pool, S: ReadableStream<Type = u8>, E> BymlConstructable<'pool, S, E> for BymlStringTableNode<'pool, S> {
  type Error = BymlStringTableNodeConstructableError<'pool, S>;

  async fn construct<F: AsyncFnOnce(u64) -> Result<BinaryReader<'pool, S>, E>>(value: u32, acquirer: F) -> Result<Result<Self, Self::Error>, E> {
    let mut reader = acquirer(value as u64).await?;

    let count: u32 = match reader.get().await {
      Ok(v) => v,
      Err(e) => return Ok(Err(BymlStringTableNodeConstructableError::AcquireReaderError(e))),
    };

    Ok(Ok(BymlStringTableNode { count, reader }))
  }
}

#[derive(FileforgeError)]
pub enum BymlStringTableNodeIntoStringError<'pool, S: ReadableStream<Type = u8>> {
  FailedToSkipToIndex(StreamSkipError<ContiguousSkipError<'pool, S::SkipError, Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, S::ReadError>>>>),
  FailedToReadOffset(StreamReadError<ArrayReadError<Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, S::ReadError>>>>),
  FailedToConsumeAddressTable(StreamSkipError<ContiguousSkipError<'pool, S::SkipError, Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, S::ReadError>>>>),

  #[report(&"OOB offset :(")]
  OobOffset,

  FailedToSkipToString(SkipError<'pool, S::SkipError>),
}

impl<'pool, S: ReadableStream<Type = u8>> BymlStringTableNode<'pool, S> {
  fn address_table_length(&self) -> u64 {
    self.count as u64 + 1
  }

  fn node_offset_to_data_offset(&self, node_offset: u32) -> Option<u32> {
    let delta = 4 + (self.address_table_length() * 4);

    node_offset.checked_sub(delta.try_into().unwrap_or(u32::MAX))
  }

  pub async fn into_string(mut self, index: u32) -> Result<ReadUntil<S>, BymlStringTableNodeIntoStringError<'pool, S>> {
    let address_table_len = self.address_table_length();
    let mut address_table = self.reader.read_ref_with::<Contiguous<_, u32, _>>(|_| {}).await.ignore();

    address_table.skip(index as u64).await.map_err(BymlStringTableNodeIntoStringError::FailedToSkipToIndex)?;

    let node_offset = address_table.read(SINGLE).await.map_err(BymlStringTableNodeIntoStringError::FailedToReadOffset)?;

    address_table.finish(address_table_len).await.map_err(BymlStringTableNodeIntoStringError::FailedToConsumeAddressTable)?;

    let data_offset = self.node_offset_to_data_offset(node_offset).ok_or(BymlStringTableNodeIntoStringError::OobOffset)?;

    self.reader.skip(data_offset as u64).await.map_err(BymlStringTableNodeIntoStringError::FailedToSkipToString)?;

    Ok(self.reader.into_stream().read_until(0))
  }
}
