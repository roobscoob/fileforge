use fileforge::{
  control_flow::ControlFlow,
  stream::{
    error::{
      stream_exhausted::StreamExhaustedError, stream_mutate::StreamMutateError, stream_overwrite::StreamOverwriteError, stream_restore::StreamRestoreError,
      stream_seek_out_of_bounds::StreamSeekOutOfBoundsError,
    },
    MutableStream, ReadableStream, ResizableStream, RestorableStream, StreamReadError, StreamSkipError,
  },
};

use crate::sead::sarc::{
  edit::SarcEditError,
  AlignmentPolicy, Sarc, SarcEntry, SarcError,
};

/// One file's data in a SARC archive, as a stream.
///
/// It can do what the archive's stream can: it's restorable if that is, can be changed in place
/// if that can, and can grow and shrink if that can, moving the files after it as the archive's
/// alignment policy says.
pub struct SarcFile<'s, 'pool, S: RestorableStream<Type = u8>, A: AlignmentPolicy> {
  sarc: &'s mut Sarc<'pool, S, A>,
  entry: SarcEntry,
  /// Where the stream is, from the start of the file.
  offset: u64,
}

impl<'s, 'pool, S: RestorableStream<Type = u8>, A: AlignmentPolicy> SarcFile<'s, 'pool, S, A> {
  pub(crate) async fn open(sarc: &'s mut Sarc<'pool, S, A>, index: u32) -> Result<Self, SarcError<'pool, S>> {
    let entry = sarc.io.entry(index).await?;
    sarc.io.seek(sarc.io.data_offset() + entry.start as u64).await?;

    Ok(SarcFile { sarc, entry, offset: 0 })
  }

  /// The file's entry, as it is now.
  pub fn entry(&self) -> &SarcEntry {
    &self.entry
  }

  fn exhausted(&self, length: u64) -> StreamExhaustedError {
    StreamExhaustedError {
      stream_length: self.entry.len(),
      read_length: length,
      read_offset: self.offset,
    }
  }

  fn fits(&self, length: u64) -> bool {
    self.offset.checked_add(length).is_some_and(|end| end <= self.entry.len())
  }
}

impl<'s, 'pool, S: RestorableStream<Type = u8>, A: AlignmentPolicy> ReadableStream for SarcFile<'s, 'pool, S, A> {
  type Type = u8;
  type ReadError = S::ReadError;
  type SkipError = S::SkipError;

  fn len(&self) -> Option<u64> {
    Some(self.entry.len())
  }

  fn offset(&self) -> u64 {
    self.offset
  }

  async fn read<const SIZE: usize, V>(&mut self, reader: impl AsyncFnOnce(&[u8; SIZE]) -> V) -> Result<V, StreamReadError<Self::ReadError>> {
    if !self.fits(SIZE as u64) {
      return Err(StreamReadError::StreamExhausted(self.exhausted(SIZE as u64)));
    }

    let value = self.sarc.io.reader.stream_mut().read(reader).await?;
    self.offset += SIZE as u64;
    Ok(value)
  }

  async fn skip(&mut self, size: u64) -> Result<(), StreamSkipError<Self::SkipError>> {
    if !self.fits(size) {
      return Err(StreamSkipError::OutOfBounds(StreamSeekOutOfBoundsError {
        stream_length: self.entry.len(),
        seek_point: self.offset.saturating_add(size),
      }));
    }

    self.sarc.io.reader.stream_mut().skip(size).await?;
    self.offset += size;
    Ok(())
  }
}

impl<'s, 'pool, S: RestorableStream<Type = u8>, A: AlignmentPolicy> RestorableStream for SarcFile<'s, 'pool, S, A> {
  type Snapshot = (S::Snapshot, u64);
  type RestoreError = S::RestoreError;

  fn snapshot(&self) -> Self::Snapshot {
    (self.sarc.io.reader.stream().snapshot(), self.offset)
  }

  async fn restore(&mut self, (snapshot, offset): Self::Snapshot) -> Result<(), StreamRestoreError<Self::RestoreError>> {
    self.sarc.io.reader.stream_mut().restore(snapshot).await?;
    self.offset = offset;
    Ok(())
  }
}

impl<'s, 'pool, S: RestorableStream<Type = u8> + MutableStream, A: AlignmentPolicy> MutableStream for SarcFile<'s, 'pool, S, A> {
  type MutateError = S::MutateError;

  async fn mutate<const SIZE: usize, V: ControlFlow>(&mut self, mutator: impl AsyncFnOnce(&mut [u8; SIZE]) -> V) -> Result<V, StreamMutateError<Self::MutateError>> {
    if !self.fits(SIZE as u64) {
      return Err(StreamMutateError::StreamExhausted(self.exhausted(SIZE as u64)));
    }

    let value = self.sarc.io.reader.stream_mut().mutate(mutator).await?;
    self.offset += SIZE as u64;
    Ok(value)
  }
}

impl<'s, 'pool, S: RestorableStream<Type = u8> + MutableStream + ResizableStream, A: AlignmentPolicy> ResizableStream for SarcFile<'s, 'pool, S, A> {
  type OverwriteError = SarcEditError<'pool, S>;

  /// Replaces `length` bytes of the file with `data`. When that changes the file's size, the files
  /// after it move, by a multiple of the largest alignment the policy gives any of them, and the
  /// padding after this file takes up the difference.
  async fn overwrite<const SIZE: usize>(&mut self, length: u64, data: [u8; SIZE]) -> Result<(), StreamOverwriteError<Self::OverwriteError>> {
    if !self.fits(length) {
      return Err(StreamOverwriteError::StreamExhausted(self.exhausted(length)));
    }

    let at = self.sarc.io.data_offset() + self.entry.start as u64 + self.offset;
    self.entry = self.sarc.resize_within(self.entry, at, length, data).await?;
    self.offset += SIZE as u64;

    self.sarc.io.seek(at + SIZE as u64).await.map_err(SarcEditError::Access)?;
    Ok(())
  }
}
