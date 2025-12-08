use crate::{
  binary_reader::{
    endianness::Endianness,
    error::{common::Read, primitive_name_annotation::PrimitiveName, GetPrimitiveError},
    readable::Readable,
    BinaryReader, PrimitiveReader,
  },
  error::ext::annotations::annotated::Annotated,
  stream::ReadableStream,
};

use super::Primitive;

impl Primitive<1> for bool {
  fn read(data: &[u8; 1], _: Endianness) -> Self {
    data[0] != 0
  }

  fn write(&self, data: &mut [u8; 1], _: Endianness) {
    data[0] = if *self { 1 } else { 0 }
  }
}

impl<'pool, S: ReadableStream<Type = u8>> Readable<'pool, S> for bool {
  type Argument = ();
  type Error = Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, S::ReadError>>;

  const SIZE: Option<u64> = Some(1);

  async fn read(reader: &mut BinaryReader<'pool, S>, _: Self::Argument) -> Result<Self, Self::Error> {
    reader.get().await
  }

  fn measure(&self) -> Option<u64> {
    Some(1)
  }
}
