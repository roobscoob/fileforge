use fileforge_macros::FileforgeError;

use crate::stream::{
  self,
  error::{stream_exhausted::StreamExhaustedError, stream_seek_out_of_bounds::StreamSeekOutOfBoundsError},
  ReadableStream, StreamReadError, StreamSkipError,
};

pub struct ReadUntil<R: ReadableStream> {
  stream: R,
  needle: R::Type,
  start_offset: u64,
  needle_offset: Option<u64>,
}

impl<R: ReadableStream> ReadUntil<R> {
  pub fn new(stream: R, needle: R::Type) -> Self {
    let start_offset = stream.offset();
    Self {
      stream,
      needle,
      start_offset,
      needle_offset: None,
    }
  }
}

#[derive(FileforgeError)]
#[report(&"TODO: Improve")]
pub struct ReadUntilSkipError<E: stream::UserReadError>(StreamReadError<E>);

impl<E: stream::UserReadError> stream::UserSkipError for ReadUntilSkipError<E> {}

impl<R: ReadableStream> ReadableStream for ReadUntil<R>
where
  R::Type: PartialEq + Copy,
{
  type Type = R::Type;
  type ReadError = R::ReadError;
  type SkipError = ReadUntilSkipError<Self::ReadError>;

  fn len(&self) -> Option<u64> {
    // If we've found the needle, length is from start to needle
    self.needle_offset.map(|n| n - self.start_offset)
  }

  fn remaining(&self) -> Option<u64> {
    match self.needle_offset {
      Some(needle_offset) => {
        let current_offset = self.stream.offset();
        Some(needle_offset.saturating_sub(current_offset))
      }
      None => None,
    }
  }

  fn offset(&self) -> u64 {
    self.stream.offset()
  }

  async fn read<const SIZE: usize, V>(&mut self, reader: impl AsyncFnOnce(&[Self::Type; SIZE]) -> V) -> Result<V, StreamReadError<Self::ReadError>> {
    let current_offset = self.stream.offset();

    // Check if this read would collide with known needle position
    if let Some(needle_offset) = self.needle_offset.filter(|&o| current_offset + SIZE as u64 > o) {
      return Err(StreamReadError::StreamExhausted(StreamExhaustedError {
        stream_length: needle_offset - self.start_offset,
        read_length: SIZE as u64,
        read_offset: current_offset - self.start_offset,
      }));
    }

    // Read from underlying stream
    let result = self
      .stream
      .read::<SIZE, _>(async |data| {
        // Check if needle is in this chunk
        for (i, &item) in data.iter().enumerate() {
          if item == self.needle {
            // Found needle at position i
            let needle_offset = current_offset + i as u64;
            self.needle_offset = Some(needle_offset);

            // Stream is exhausted - can't provide SIZE elements before needle
            return Err(StreamReadError::StreamExhausted(StreamExhaustedError {
              stream_length: needle_offset - self.start_offset,
              read_length: SIZE as u64,
              read_offset: current_offset - self.start_offset,
            }));
          }
        }

        // No needle found in this chunk, pass through
        Ok(reader(data).await)
      })
      .await??;

    Ok(result)
  }

  async fn skip(&mut self, size: u64) -> Result<(), StreamSkipError<ReadUntilSkipError<Self::ReadError>>> {
    let current_offset = self.stream.offset();

    // Check if this skip would collide with known needle position
    if let Some(needle_offset) = self.needle_offset.filter(|&o| current_offset + size > o) {
      return Err(StreamSkipError::OutOfBounds(StreamSeekOutOfBoundsError {
        stream_length: needle_offset - self.start_offset,
        seek_point: (current_offset - self.start_offset) + size,
      }));
    }

    // We need to check for needle while skipping
    // Read one element at a time
    for _ in 0..size {
      let offset = self.stream.offset();
      match self.stream.read::<1, _>(async |data| data[0] == self.needle).await {
        Ok(true) => {
          // Found needle during skip
          self.needle_offset = Some(offset);
          return Err(StreamSkipError::OutOfBounds(StreamSeekOutOfBoundsError {
            stream_length: offset - self.start_offset,
            seek_point: (current_offset - self.start_offset) + size,
          }));
        }
        Ok(false) => {}
        Err(e) => return Err(StreamSkipError::User(ReadUntilSkipError(e))),
      }
    }

    Ok(())
  }
}
