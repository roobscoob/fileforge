use crate::{
  binary_reader::{
    error::{common::Read, primitive_name_annotation::PrimitiveName, GetPrimitiveError},
    readable::Readable,
    BinaryReader, PrimitiveReader,
  },
  error::ext::annotations::annotated::Annotated,
  stream::ReadableStream,
};

use super::Primitive;

impl Primitive<0> for () {
  fn read(_: &[u8; 0], _: crate::binary_reader::endianness::Endianness) -> Self {}
  fn write(&self, _: &mut [u8; 0], _: crate::binary_reader::endianness::Endianness) {}
}

impl<'pool, S: ReadableStream<Type = u8>> Readable<'pool, S> for () {
  type Argument = ();
  type Error = Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, S::ReadError>>;

  const SIZE: Option<u64> = Some(0);

  async fn read(reader: &mut BinaryReader<'pool, S>, _: Self::Argument) -> Result<Self, Self::Error> {
    reader.get().await
  }

  fn measure(&self) -> Option<u64> {
    Some(0)
  }
}
