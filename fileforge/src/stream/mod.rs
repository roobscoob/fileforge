pub mod builtin;
pub mod collectable;
pub mod error;
pub mod extensions;

use collectable::Collectable;
use error::{
  stream_mutate::StreamMutateError, stream_overwrite::StreamOverwriteError, stream_partition::StreamPartitionError, stream_read::StreamReadError, stream_rewind::StreamRewindError,
  stream_seek::StreamSeekError, stream_skip::StreamSkipError, user_mutate::UserMutateError, user_overwrite::UserOverwriteError, user_partition::UserPartitionError, user_read::UserReadError,
  user_rewind::UserRewindError, user_seek::UserSeekError, user_skip::UserSkipError,
};

use crate::{
  control_flow::ControlFlow,
  stream::error::{stream_restore::StreamRestoreError, user_restore::UserRestoreError},
};

#[allow(non_snake_case)]
pub async fn SINGLE<T>(v: &[T; 1]) -> T
where
  T: Copy,
{
  v[0]
}

#[allow(non_snake_case)]
pub async fn DOUBLE<T>(v: &[T; 2]) -> (T, T)
where
  T: Copy,
{
  (v[0], v[1])
}

#[allow(non_snake_case)]
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

  async fn mutate<const SIZE: usize, V: ControlFlow>(&mut self, mutator: impl AsyncFnOnce(&mut [Self::Type; SIZE]) -> V) -> Result<V, StreamMutateError<Self::MutateError>>;
}

impl<Substream: MutableStream> MutableStream for &mut Substream {
  type MutateError = Substream::MutateError;

  async fn mutate<const SIZE: usize, V: ControlFlow>(&mut self, mutator: impl AsyncFnOnce(&mut [Self::Type; SIZE]) -> V) -> Result<V, StreamMutateError<Self::MutateError>> {
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

pub trait StaticPartitionableStream<const PARTITION_SIZE: usize>: ReadableStream {
  type PartitionError: UserPartitionError;
  type PartitionLeft: ReadableStream<Type = Self::Type>;
  type PartitionRight: ReadableStream<Type = Self::Type>;

  async fn partition(self) -> Result<(Self::PartitionLeft, Self::PartitionRight), StreamPartitionError<Self::PartitionError>>;
}

pub trait DynamicPartitionableStream: ReadableStream {
  type PartitionError: UserPartitionError;
  type PartitionDynamicLeft: ReadableStream<Type = Self::Type>;
  type PartitionDynamicRight: ReadableStream<Type = Self::Type>;

  async fn partition_dynamic(self, size: u64) -> Result<(Self::PartitionDynamicLeft, Self::PartitionDynamicRight), StreamPartitionError<Self::PartitionError>>;
}

pub trait RestorableStream: ReadableStream {
  type Snapshot: Clone;
  type RestoreError: UserRestoreError;

  fn snapshot(&self) -> Self::Snapshot;

  /// allows you to 'restore' a stream to a snapshot, effectively seeking to that snapshot
  /// restrictions:
  ///  - you can only 'restore' *backwards* - you can't restore forwards
  async fn restore(&mut self, snapshot: Self::Snapshot) -> Result<(), StreamRestoreError<Self::RestoreError>>;
}

impl<Substream: RestorableStream> RestorableStream for &mut Substream {
  type Snapshot = Substream::Snapshot;
  type RestoreError = Substream::RestoreError;

  fn snapshot(&self) -> Self::Snapshot {
    (**self).snapshot()
  }

  async fn restore(&mut self, snapshot: Self::Snapshot) -> Result<(), StreamRestoreError<Self::RestoreError>> {
    (**self).restore(snapshot).await
  }
}
