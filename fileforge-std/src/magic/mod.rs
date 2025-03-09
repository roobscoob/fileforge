use error::{error::MagicError, invalid::MagicInvalid};
use fileforge_lib::{reader::{readable::Readable, PrimitiveReader, Reader}, stream::ReadableStream};

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

impl<'pool: 'l, 'l, const SIZE: usize, S: ReadableStream + 'l> Readable<'pool, 'l, S> for Magic<SIZE> {
  type Error = MagicError<'pool, SIZE, S::ReadError>;
  type Argument = Magic<SIZE>;

  async fn read(reader: &'l mut Reader<'pool, S>, argument: Self::Argument) -> Result<Self, Self::Error> {
    let content = Self::from_bytes(reader.get::<[u8; SIZE]>().await?);

    MagicInvalid::assert(content, argument, || {
      reader.create_physical_diagnostic(-(SIZE as i64), Some(SIZE as u64), "Magic")
    })?;

    Ok(content)
  }
}
