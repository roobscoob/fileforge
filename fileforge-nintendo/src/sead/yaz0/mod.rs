use fileforge_lib::stream::{
  error::{stream_exhausted::StreamExhaustedError, stream_read::StreamReadError, stream_seek_out_of_bounds::StreamSeekOutOfBoundsError, stream_skip::StreamSkipError},
  ReadableStream, SINGLE,
};

use crate::sead::yaz0::{
  error::Yaz0Error,
  header::Yaz0Header,
  parser::{NoSnapshotData, Yaz0Parser},
  state::Yaz0State,
};

pub mod error;
pub mod header;
pub mod parser;
pub mod readable;
pub mod state;

pub struct Yaz0Stream<UnderlyingStream: ReadableStream<Type = u8>> {
  header: Yaz0Header,
  stream: Yaz0Parser<UnderlyingStream, NoSnapshotData>,
  state: Yaz0State,
}

impl<S: ReadableStream<Type = u8>> ReadableStream for Yaz0Stream<S> {
  type Type = u8;

  type ReadError = Yaz0Error<S::ReadError>;
  type SkipError = Yaz0Error<S::ReadError>;

  fn len(&self) -> Option<u64> {
    Some(self.header.decompressed_size().into())
  }

  fn offset(&self) -> u64 {
    self.state.offset()
  }

  async fn read<const SIZE: usize, V>(&mut self, reader: impl AsyncFnOnce(&[Self::Type; SIZE]) -> V) -> Result<V, StreamReadError<Self::ReadError>> {
    let read_offset = self.offset();
    let mut buffer = heapless::Vec::<u8, SIZE>::new();

    buffer.extend(self.state.take(buffer.capacity() - buffer.len()));

    while self.offset() < self.header.decompressed_size() as u64 && !buffer.is_full() {
      let operation = self.stream.read(SINGLE).await.map_err(|e| match e {
        StreamReadError::StreamExhausted(e) => StreamReadError::StreamExhausted(StreamExhaustedError {
          read_length: SIZE as u64,
          read_offset,
          stream_length: self.header.decompressed_size() as u64,
        }),
        StreamReadError::User(u) => StreamReadError::User(Yaz0Error::ParseError(StreamReadError::User(u))),
      })?;

      self.state.feed(operation).map_err(|e| Yaz0Error::MalformedStream(e))?;
      buffer.extend(self.state.take(buffer.capacity() - buffer.len()));
    }

    if !buffer.is_full() {
      Err(StreamExhaustedError {
        read_length: SIZE as u64,
        read_offset,
        stream_length: self.header.decompressed_size() as u64,
      })?
    }

    Ok(reader(&buffer.into_array::<SIZE>().unwrap()).await)
  }

  async fn skip(&mut self, mut read_length: u64) -> Result<(), StreamSkipError<Self::SkipError>> {
    let read_offset = self.offset();
    let original_read_length = read_length;

    read_length -= self.state.take(read_length as usize).len() as u64;

    while self.offset() < self.header.decompressed_size() as u64 && read_length != 0 {
      let operation = self.stream.read(SINGLE).await.map_err(|e| match e {
        StreamReadError::StreamExhausted(e) => StreamSkipError::OutOfBounds(StreamSeekOutOfBoundsError {
          seek_point: read_offset + original_read_length,
          stream_length: self.header.decompressed_size() as u64,
        }),
        StreamReadError::User(u) => StreamSkipError::User(Yaz0Error::ParseError(StreamReadError::User(u))),
      })?;

      self.state.feed(operation).map_err(|e| Yaz0Error::MalformedStream(e))?;
      read_length -= self.state.take(read_length as usize).len() as u64;
    }

    if read_length != 0 {
      Err(StreamSeekOutOfBoundsError {
        seek_point: read_offset + original_read_length,
        stream_length: self.header.decompressed_size() as u64,
      })?
    }

    Ok(())
  }
}
