pub mod builtin;
pub mod collectable;
pub mod error;

use collectable::Collectable;
use error::{
  stream_mutate::StreamMutateError, stream_overwrite::StreamOverwriteError, stream_partition::StreamPartitionError, stream_read::StreamReadError, stream_rewind::StreamRewindError,
  stream_seek::StreamSeekError, stream_skip::StreamSkipError, user_mutate::UserMutateError, user_overwrite::UserOverwriteError, user_partition::UserPartitionError, user_read::UserReadError,
  user_rewind::UserRewindError, user_seek::UserSeekError, user_skip::UserSkipError,
};

use crate::stream::error::user_restore::UserRestoreError;

pub async fn SINGLE<T>(v: &[T; 1]) -> T
where
  T: Copy,
{
  v[0]
}

pub async fn DOUBLE<T>(v: &[T; 2]) -> (T, T)
where
  T: Copy,
{
  (v[0], v[1])
}

pub async fn CLONED<T>(v: &[T; 1]) -> T
where
  T: Clone,
{
  v[0].clone()
}

pub trait ReadableStream: Sized {
  type Type;

  type ReadError: UserReadError;
  type SkipError: UserSkipError;

  fn len(&self) -> Option<u64> {
    return None;
  }
  fn remaining(&self) -> Option<u64> {
    Some(self.len()? - self.offset())
  }
  fn offset(&self) -> u64;

  async fn read<const SIZE: usize, V>(&mut self, reader: impl AsyncFnOnce(&[Self::Type; SIZE]) -> V) -> Result<V, StreamReadError<Self::ReadError>>;
  async fn skip(&mut self, size: u64) -> Result<(), StreamSkipError<Self::SkipError>>;

  async fn collect<C: Collectable<Self>>(&mut self, mut collector: C) -> Result<C, C::Error> {
    collector.collect(self).await?;

    Ok(collector)
  }
}

impl<Substream: ReadableStream> ReadableStream for &mut Substream {
  type Type = Substream::Type;
  type ReadError = Substream::ReadError;
  type SkipError = Substream::SkipError;

  fn len(&self) -> Option<u64> {
    (**self).len()
  }
  fn remaining(&self) -> Option<u64> {
    (**self).remaining()
  }
  fn offset(&self) -> u64 {
    (**self).offset()
  }
  async fn read<const SIZE: usize, V>(&mut self, reader: impl AsyncFnOnce(&[Self::Type; SIZE]) -> V) -> Result<V, StreamReadError<Self::ReadError>> {
    (**self).read(reader).await
  }
  async fn skip(&mut self, size: u64) -> Result<(), StreamSkipError<Self::SkipError>> {
    (**self).skip(size).await
  }
}

pub trait RewindableStream: ReadableStream {
  type RewindError: UserRewindError;

  async fn rewind(&mut self, size: u64) -> Result<(), StreamRewindError<Self::RewindError>>;
}

impl<Substream: RewindableStream> RewindableStream for &mut Substream {
  type RewindError = Substream::RewindError;

  async fn rewind(&mut self, size: u64) -> Result<(), StreamRewindError<Self::RewindError>> {
    (**self).rewind(size).await
  }
}

pub trait SeekableStream: RewindableStream {
  type SeekError: UserSeekError;

  async fn seek(&mut self, offset: u64) -> Result<(), StreamSeekError<Self::SeekError>>;
}

impl<Substream: SeekableStream> SeekableStream for &mut Substream {
  type SeekError = Substream::SeekError;

  async fn seek(&mut self, offset: u64) -> Result<(), StreamSeekError<Self::SeekError>> {
    (**self).seek(offset).await
  }
}

pub trait MutableStream: ReadableStream {
  type MutateError: UserMutateError;

  async fn mutate<const SIZE: usize, V>(&mut self, mutator: impl AsyncFnOnce(&mut [Self::Type; SIZE]) -> V) -> Result<V, StreamMutateError<Self::MutateError>>;
}

impl<Substream: MutableStream> MutableStream for &mut Substream {
  type MutateError = Substream::MutateError;

  async fn mutate<const SIZE: usize, V>(&mut self, mutator: impl AsyncFnOnce(&mut [Self::Type; SIZE]) -> V) -> Result<V, StreamMutateError<Self::MutateError>> {
    (**self).mutate(mutator).await
  }
}

pub trait ResizableStream: ReadableStream {
  type OverwriteError: UserOverwriteError;

  async fn overwrite<const SIZE: usize>(&mut self, length: u64, data: [Self::Type; SIZE]) -> Result<(), StreamOverwriteError<Self::OverwriteError>>;
}

impl<Substream: ResizableStream> ResizableStream for &mut Substream {
  type OverwriteError = Substream::OverwriteError;

  async fn overwrite<const SIZE: usize>(&mut self, length: u64, data: [Self::Type; SIZE]) -> Result<(), StreamOverwriteError<Self::OverwriteError>> {
    (**self).overwrite(length, data).await
  }
}

pub trait StaticPartitionableStream<'l, const PARTITION_SIZE: usize>: ReadableStream {
  type PartitionError: UserPartitionError;
  type Partition: ReadableStream<Type = Self::Type> + 'l;

  async fn partition(&'l mut self) -> Result<Self::Partition, StreamPartitionError<Self::PartitionError>>;
}

impl<'l, const PARTITION_SIZE: usize, Substream: StaticPartitionableStream<'l, PARTITION_SIZE>> StaticPartitionableStream<'l, PARTITION_SIZE> for &mut Substream {
  type PartitionError = Substream::PartitionError;
  type Partition = Substream::Partition;

  async fn partition(&'l mut self) -> Result<Self::Partition, StreamPartitionError<Self::PartitionError>> {
    (**self).partition().await
  }
}

pub trait DynamicPartitionableStream<'l>: ReadableStream {
  type PartitionError: UserPartitionError;
  type PartitionDynamic: ReadableStream<Type = Self::Type>;

  async fn partition_dynamic(&'l mut self, size: u64) -> Result<Self::PartitionDynamic, StreamPartitionError<Self::PartitionError>>;
}

impl<'l, Substream: DynamicPartitionableStream<'l>> DynamicPartitionableStream<'l> for &mut Substream {
  type PartitionError = Substream::PartitionError;
  type PartitionDynamic = Substream::PartitionDynamic;

  async fn partition_dynamic(&'l mut self, size: u64) -> Result<Self::PartitionDynamic, StreamPartitionError<Self::PartitionError>> {
    (**self).partition_dynamic(size).await
  }
}

enum DefaultPeekError<Read, Restore> {
  ReadFailed(Read),
  RestoreFailed(Restore),
}

pub trait RestorableStream: ReadableStream {
  type Snapshot;
  type RestoreError: UserRestoreError;
  type PeekError = DefaultPeekError<StreamReadError<Self::ReadError>, Self::RestoreError>

  fn snapshot(&self) -> Self::Snapshot;

  /// allows you to 'restore' a stream to a snapshot, effectively seeking to that snapshot
  /// restrictions:
  ///  - you can only 'restore' *backwards* - you can't restore forwards
  async fn restore(&mut self, snapshot: Self::Snapshot) -> Result<(), Self::RestoreError>;

  async fn peek<const SIZE: usize, V>(&mut self, reader: impl AsyncFnOnce(&[Self::Type; SIZE]) -> V) -> Result<V, StreamReadError<Self::ReadError>> {
    let s = self.snapshot().await;

    self.read(reader)
  }
}

impl<Substream: RestorableStream> RestorableStream for &mut Substream {
  type Snapshot = Substream::Snapshot;
  type RestoreError = Substream::RestoreError;

  fn snapshot(&self) -> Self::Snapshot {
    (**self).snapshot()
  }

  async fn restore(&mut self, snapshot: Self::Snapshot) -> Result<(), Self::RestoreError> {
    (**self).restore(snapshot).await
  }
}
