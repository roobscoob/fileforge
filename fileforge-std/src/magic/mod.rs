use error::MagicError;
use fileforge_lib::{reader::{readable::{error::readable::ReadableError, Readable}, PrimitiveReader, Reader}, stream::ReadableStream};

pub mod renderable;
pub mod error;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Magic<const SIZE: usize> {
  bytes: [u8; SIZE],
}

impl<const SIZE: usize> Magic<SIZE> {
  pub const fn from_bytes(bytes: [u8; SIZE]) -> Magic<SIZE> { Self { bytes } }
  pub const fn from_byte_ref(bytes: &[u8; SIZE]) -> Magic<SIZE> { Self { bytes: *bytes } }
}

impl<'pool, 'l, const NODE_NAME_SIZE: usize, const SIZE: usize> Readable<'pool, 'l, NODE_NAME_SIZE> for Magic<SIZE> {
  type Error<S: ReadableStream<NODE_NAME_SIZE>> = MagicError<'pool, NODE_NAME_SIZE, SIZE> where S: Sized, 'pool: 'l, S: 'l;
  type Argument = Magic<SIZE>;

  async fn read<S: ReadableStream<NODE_NAME_SIZE>>(reader: &'l mut Reader<'pool, NODE_NAME_SIZE, S>, argument: Self::Argument) -> Result<Self, ReadableError<'pool, NODE_NAME_SIZE, Self::Error<S>, S::ReadError>> {
    let content = Self::from_bytes(reader.get::<[u8; SIZE]>().await?);

    MagicError::assert(content, argument, || {
      reader.create_physical_diagnostic(-4, Some(4), "Magic")
    })?;

    Ok(content)
  }
}
