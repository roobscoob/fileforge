use core::convert::Infallible;

use fileforge::{
  binary_reader::{
    readable::{IntoReadable, Readable},
    BinaryReader,
  },
  stream::ReadableStream,
};

use crate::byml::{
  header::{BymlHeader, BymlHeaderConfig},
  Byml,
};

impl<'pool, S: ReadableStream<Type = u8>> IntoReadable<'pool, S> for Byml<'pool, S> {
  type Argument = BymlHeaderConfig;
  type Error = <BymlHeader as Readable<'pool, S>>::Error;

  async fn read(mut reader: BinaryReader<'pool, S>, argument: Self::Argument) -> Result<Self, Self::Error> {
    let header: BymlHeader = reader.read_with(argument).await?;

    Ok(Self { header, reader })
  }
}
