use fileforge::{
  binary_reader::error::{common::SeekOffset, SkipError},
  stream::{
    error::{stream_read::StreamReadError, stream_seek_out_of_bounds::StreamSeekOutOfBoundsError, stream_skip::StreamSkipError},
    ReadableStream,
  },
};

use crate::sead::sarc::sfat::{
  entry::{SfatEntry, SfatEntryError, SFAT_ENTRY_SIZE},
  stream::SfatStream,
};

impl<'pool, Underlying: ReadableStream<Type = u8>> ReadableStream for SfatStream<'pool, Underlying> {
  type Type = SfatEntry;

  type ReadError = SfatEntryError<'pool, Underlying::ReadError>;
  type SkipError = Underlying::SkipError;

  fn offset(&self) -> u64 {
    self.stream.offset() / SFAT_ENTRY_SIZE
  }

  async fn skip(&mut self, size: u64) -> Result<(), StreamSkipError<Self::SkipError>> {
    self.stream.skip(size * SFAT_ENTRY_SIZE).await.map_err(|e| match e {
      SkipError::User(u) => StreamSkipError::User(u),
      SkipError::OutOfBounds(oob) => match oob.seek_offset {
        SeekOffset::InBounds(offset) => StreamSkipError::OutOfBounds(StreamSeekOutOfBoundsError {
          stream_length: oob.provider_size.value() / SFAT_ENTRY_SIZE,
          seek_point: offset / SFAT_ENTRY_SIZE,
        }),
        _ => panic!("The overflow and underflow states don't really make sense atm."),
      },
    })
  }

  async fn read<const SIZE: usize, V>(&mut self, reader: impl AsyncFnOnce(&[Self::Type; SIZE]) -> V) -> Result<V, StreamReadError<Self::ReadError>> {
    let mut dest = heapless::Vec::<SfatEntry, SIZE>::new();

    while !dest.is_full() {
      dest.push(self.stream.read().await?).map_err(|_| {}).unwrap();
    }

    Ok(reader(&dest.into_array::<SIZE>().map_err(|_| {}).unwrap()).await)
  }
}
