use crate::{error::FileforgeError, stream::ReadableStream};

use super::BinaryReader;

pub trait Readable<'pool, S: ReadableStream<Type = u8>>: Sized {
  type Error: FileforgeError;
  type Argument;

  async fn read(reader: &mut BinaryReader<'pool, S>, argument: Self::Argument) -> Result<Self, Self::Error>;
}

pub trait IntoReadable<'pool, S: ReadableStream<Type = u8>>: Sized {
  type Error: FileforgeError;
  type Argument;

  async fn read(reader: BinaryReader<'pool, S>, argument: Self::Argument) -> Result<Self, Self::Error>;
}

pub trait NoneArgument {
  fn none() -> Self;
}

impl NoneArgument for () {
  fn none() -> Self {
    ()
  }
}
