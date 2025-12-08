use fileforge::{
  encoding::Decoder,
  stream::{ReadableStream, StreamReadError, UserReadError},
};
use fileforge_macros::FileforgeError;

use crate::encodings::ascii::codepages::AsciiCodepage;

pub struct AsciiDecoder<Codepage: AsciiCodepage, S: ReadableStream<Type = char>>(Codepage, S);

#[derive(FileforgeError, Debug)]
pub enum AsciiDecodeError<E: UserReadError, Codepage: AsciiCodepage> {
  #[report(&"todo: change")]
  User(E),
  Codepage(Codepage::DecodeError),
}

impl<Codepage: AsciiCodepage, S: UserReadError> UserReadError for AsciiDecodeError<S, Codepage> {}

impl<Codepage: AsciiCodepage, S: ReadableStream<Type = char>> Decoder<S> for AsciiDecoder<Codepage, S> {
  fn new(input: S) -> Self {
    AsciiDecoder(Codepage::default(), input)
  }
}

impl<Codepage: AsciiCodepage, S: ReadableStream<Type = char>> ReadableStream for AsciiDecoder<Codepage, S> {
  type ReadError = AsciiDecodeError<S::ReadError, Codepage>;
  type SkipError = S::SkipError;
  type Type = u8;

  fn len(&self) -> Option<u64> {
    self.1.len()
  }

  fn offset(&self) -> u64 {
    self.1.offset()
  }

  fn remaining(&self) -> Option<u64> {
    self.1.remaining()
  }

  async fn read<const SIZE: usize, V>(&mut self, reader: impl AsyncFnOnce(&[Self::Type; SIZE]) -> V) -> Result<V, StreamReadError<Self::ReadError>> {
    match self
      .1
      .read(async |values: &[char; SIZE]| {
        let mut bytes: [u8; SIZE] = [0; SIZE];

        for (i, &item) in values.iter().enumerate() {
          bytes[i] = if (item as u32) < 128 { item as u8 } else { self.0.from_char(item)? };
        }

        Ok(reader(&bytes).await)
      })
      .await
    {
      Ok(Ok(v)) => Ok(v),
      Ok(Err(e)) => Err(StreamReadError::User(AsciiDecodeError::Codepage(e))),
      Err(StreamReadError::User(e)) => Err(StreamReadError::User(AsciiDecodeError::User(e))),
      Err(StreamReadError::StreamExhausted(e)) => Err(StreamReadError::StreamExhausted(e)),
    }
  }

  async fn skip(&mut self, size: u64) -> Result<(), fileforge::stream::StreamSkipError<Self::SkipError>> {
    self.1.skip(size).await
  }
}
