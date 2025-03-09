use error::{error::ByteOrderMarkError, invalid::ByteOrderMarkInvalid};
use fileforge_lib::{reader::{endianness::Endianness, readable::Readable, PrimitiveReader, Reader}, stream::ReadableStream};

pub mod renderable;
pub mod error;

#[derive(PartialEq, Eq, Clone, Copy)]
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

  pub fn be(&self) -> Self {
    match self.endianness {
      Endianness::BigEndian => *self,
      Endianness::LittleEndian => Self { endianness: Endianness::LittleEndian, bytes: [self.bytes[1], self.bytes[0]] },
    }
  }

  pub fn le(&self) -> Self {
    match self.endianness {
      Endianness::BigEndian => Self { endianness: Endianness::BigEndian, bytes: [self.bytes[1], self.bytes[0]] },
      Endianness::LittleEndian => *self,
    }
  }

  pub fn bytes(&self) -> [u8; 2] {
    self.bytes
  }
}

impl<'pool: 'l, 'l, S: ReadableStream + 'l> Readable<'pool, 'l, S> for ByteOrderMark {
  type Error = ByteOrderMarkError<'pool, S::ReadError>;
  type Argument = ByteOrderMark;

  async fn read(reader: &'l mut Reader<'pool, S>, argument: Self::Argument) -> Result<Self, Self::Error> {
    let bytes = reader.get::<[u8; 2]>().await?;
    let content = ByteOrderMark::from_bytes(reader.get_endianness(), bytes);

    ByteOrderMarkInvalid::assert(argument, content, || {
      reader.create_physical_diagnostic(-2, Some(2), "ByteOrderMark")
    })?;

    Ok(content)
  }
}