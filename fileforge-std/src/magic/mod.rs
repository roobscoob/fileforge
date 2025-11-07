use error::invalid::MagicInvalid;
use fileforge::{
  binary_reader::{readable::Readable, BinaryReader, PrimitiveReader},
  stream::ReadableStream,
};

use crate::magic::error::MagicError;

pub mod error;
pub mod renderable;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Magic<const SIZE: usize> {
  bytes: [u8; SIZE],
}

impl<const SIZE: usize> Magic<SIZE> {
  pub const fn from_bytes(bytes: [u8; SIZE]) -> Magic<SIZE> {
    Self { bytes }
  }
  pub const fn from_byte_ref(bytes: &[u8; SIZE]) -> Magic<SIZE> {
    Self { bytes: *bytes }
  }
}

impl<'pool, const SIZE: usize, S: ReadableStream<Type = u8>> Readable<'pool, S> for Magic<SIZE> {
  type Error = MagicError<'pool, SIZE, S::ReadError>;
  type Argument = Magic<SIZE>;

  async fn read(reader: &mut BinaryReader<'pool, S>, argument: Self::Argument) -> Result<Self, Self::Error> {
    let content = Self::from_bytes(reader.get::<[u8; SIZE]>().await?);

    MagicInvalid::assert(content, argument, || reader.create_physical_diagnostic(-(SIZE as i128), Some(SIZE as u64), "Magic"))?;

    Ok(content)
  }
}
