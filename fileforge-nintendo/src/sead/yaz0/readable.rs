use fileforge_lib::{
  binary_reader::{readable::IntoReadable, BinaryReader},
  stream::ReadableStream,
};

use crate::sead::yaz0::{header::readable::Yaz0HeaderReadError, parser::Yaz0Parser, state::Yaz0State, Yaz0Stream};

impl<'pool, S: ReadableStream<Type = u8>> IntoReadable<'pool, S> for Yaz0Stream<S> {
  type Argument = ();
  type Error = Yaz0HeaderReadError<'pool, S::ReadError>;

  async fn read(mut reader: BinaryReader<'pool, S>, _: Self::Argument) -> Result<Self, Self::Error> {
    Ok(Yaz0Stream {
      header: reader.read().await?,
      state: Yaz0State::empty(),
      stream: Yaz0Parser::new(reader.into_stream()),
    })
  }
}
