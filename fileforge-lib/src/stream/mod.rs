pub mod builtin;
pub mod collectable;
pub mod error;

use core::future::Future;

use collectable::Collectable;
use error::{
  stream_mutate::StreamMutateError, stream_overwrite::StreamOverwriteError, stream_partition::StreamPartitionError, stream_read::StreamReadError, stream_rewind::StreamRewindError,
  stream_seek::StreamSeekError, stream_skip::StreamSkipError, stream_write::StreamWriteError, user_mutate::UserMutateError, user_overwrite::UserOverwriteError, user_partition::UserPartitionError,
  user_read::UserReadError, user_rewind::UserRewindError, user_seek::UserSeekError, user_skip::UserSkipError, user_write::UserWriteError,
};

pub trait ReadableStream<const NODE_NAME_SIZE: usize>: Sized {
  type ReadError: UserReadError<NODE_NAME_SIZE>;

  fn len(&self) -> Option<u64> { return None }
  fn remaining(&self) -> Option<u64> { return None }
  fn offset(&self) -> u64;

  fn read<const SIZE: usize, V, R: Future<Output = V>>(&mut self, reader: impl FnOnce(&[u8; SIZE]) -> R) -> impl Future<Output = Result<V, StreamReadError<NODE_NAME_SIZE, Self::ReadError>>>;

  fn collect<C: Collectable<NODE_NAME_SIZE, Self>>(&mut self, mut collector: C) -> impl Future<Output = Result<C, C::Error>> {
    async {
      collector.collect(self).await?;

      Ok(collector)
    }
  }
}

pub trait SkippableStream<const NODE_NAME_SIZE: usize>: ReadableStream<NODE_NAME_SIZE> {
  type SkipError: UserSkipError<NODE_NAME_SIZE>;

  fn skip(&mut self, size: u64) -> impl Future<Output = Result<(), StreamSkipError<NODE_NAME_SIZE, Self::SkipError>>>;
}

pub trait RewindableStream<const NODE_NAME_SIZE: usize>: ReadableStream<NODE_NAME_SIZE> {
  type RewindError: UserRewindError<NODE_NAME_SIZE>;

  fn rewind(&mut self, size: u64) -> impl Future<Output = Result<(), StreamRewindError<NODE_NAME_SIZE, Self::RewindError>>>;
}

pub trait SeekableStream<const NODE_NAME_SIZE: usize>: ReadableStream<NODE_NAME_SIZE> {
  type SeekError: UserSeekError<NODE_NAME_SIZE>;

  fn seek(&mut self, offset: u64) -> impl Future<Output = Result<(), StreamSeekError<NODE_NAME_SIZE, Self::SeekError>>>;
}

pub trait MutableStream<const NODE_NAME_SIZE: usize>: ReadableStream<NODE_NAME_SIZE> {
  type MutateError: UserMutateError<NODE_NAME_SIZE>;

  fn mutate<const SIZE: usize, V, R: Future<Output = V>>(
    &mut self,
    mutator: impl FnOnce(&mut [u8; SIZE]) -> R,
  ) -> impl Future<Output = Result<V, StreamMutateError<NODE_NAME_SIZE, Self::MutateError>>>;
}

pub trait ResizableStream<const NODE_NAME_SIZE: usize>: ReadableStream<NODE_NAME_SIZE> {
  type OverwriteError: UserOverwriteError<NODE_NAME_SIZE>;

  fn overwrite<const SIZE: usize>(&mut self, length: u64, data: &[u8; SIZE]) -> impl Future<Output = Result<(), StreamOverwriteError<NODE_NAME_SIZE, Self::OverwriteError>>>;
}

pub trait StaticPartitionableStream<'l, const NODE_NAME_SIZE: usize, const PARTITION_SIZE: usize>: ReadableStream<NODE_NAME_SIZE> {
  type PartitionError: UserPartitionError<NODE_NAME_SIZE>;
  type Partition: ReadableStream<NODE_NAME_SIZE> + 'l;

  fn partition(&'l mut self) -> impl Future<Output = Result<Self::Partition, StreamPartitionError<NODE_NAME_SIZE, Self::PartitionError>>>;
}

pub trait DynamicPartitionableStream<'l, const NODE_NAME_SIZE: usize>: ReadableStream<NODE_NAME_SIZE> {
  type PartitionError: UserPartitionError<NODE_NAME_SIZE>;
  type PartitionDynamic: ReadableStream<NODE_NAME_SIZE>;

  fn partition_dynamic(&'l mut self, size: u64) -> impl Future<Output = Result<Self::PartitionDynamic, StreamPartitionError<NODE_NAME_SIZE, Self::PartitionError>>>;
}

pub trait WritableStream<const NODE_NAME_SIZE: usize> {
  type WriteError: UserWriteError<NODE_NAME_SIZE>;

  fn write<const SIZE: usize, V, R: Future<Output = V>>(&mut self, writer: impl FnOnce(&mut [u8; SIZE]) -> R) -> impl Future<Output = Result<V, StreamWriteError<NODE_NAME_SIZE, Self::WriteError>>>;
}
