use error::invalid::ByteOrderMarkInvalid;
use fileforge::{
  binary_reader::{endianness::Endianness, readable::Readable, BinaryReader, PrimitiveReader},
  stream::ReadableStream,
};

use crate::byte_order_mark::error::ByteOrderMarkError;

pub mod error;
pub mod renderable;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ByteOrderMark {
  endianness: Endianness,
  bytes: [u8; 2],
}

impl ByteOrderMark {
  pub const fn from_bytes(endianness: Endianness, bytes: [u8; 2]) -> ByteOrderMark {
    Self { endianness, bytes }
  }

  pub const fn from_byte_ref(endianness: Endianness, bytes: &[u8; 2]) -> ByteOrderMark {
    Self::from_bytes(endianness, *bytes)
  }

  pub fn endianness(&self) -> Endianness {
    self.endianness
  }

  pub fn bytes(&self) -> [u8; 2] {
    self.bytes
  }

  pub fn swap(self) -> Self {
    ByteOrderMark {
      endianness: self.endianness.swap(),
      bytes: [self.bytes[1], self.bytes[0]],
    }
  }
}

impl<'pool, S: ReadableStream<Type = u8>> Readable<'pool, S> for ByteOrderMark {
  type Error = ByteOrderMarkError<'pool, S::ReadError>;
  type Argument = ByteOrderMark;

  async fn read(reader: &mut BinaryReader<'pool, S>, argument: Self::Argument) -> Result<Self, Self::Error> {
    let bytes = reader.get::<[u8; 2]>().await?;

    let endianness = ByteOrderMarkInvalid::assert(argument, bytes, || reader.create_physical_diagnostic(-2, Some(2), "ByteOrderMark"))?;

    Ok(Self::from_bytes(endianness, bytes))
  }
}
