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

pub trait ReadableStream: Sized {
  type ReadError: UserReadError;
  type SkipError: UserSkipError;

  fn len(&self) -> Option<u64> { return None }
  fn remaining(&self) -> Option<u64> { Some(self.len()? - self.offset()) }
  fn offset(&self) -> u64;

  async fn read<const SIZE: usize, V>(&mut self, reader: impl AsyncFnOnce(&[u8; SIZE]) -> V) -> Result<V, StreamReadError<Self::ReadError>>;
  async fn skip(&mut self, size: u64) -> Result<(), StreamSkipError<Self::SkipError>>;

  async fn collect<C: Collectable<Self>>(&mut self, mut collector: C) -> Result<C, C::Error> {
    collector.collect(self).await?;

    Ok(collector)
  }
}

impl<Substream: ReadableStream> ReadableStream for &mut Substream {
  type ReadError = Substream::ReadError;
  type SkipError = Substream::SkipError;

  fn len(&self) -> Option<u64> { (**self).len() }
  fn remaining(&self) -> Option<u64> { (**self).remaining() }
  fn offset(&self) -> u64 { (**self).offset() }
  async fn read<const SIZE: usize, V>(&mut self, reader: impl AsyncFnOnce(&[u8; SIZE]) -> V) -> Result<V, StreamReadError<Self::ReadError>> {
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

  async fn mutate<const SIZE: usize, V>(
    &mut self,
    mutator: impl AsyncFnOnce(&mut [u8; SIZE]) -> V,
  ) -> Result<V, StreamMutateError<Self::MutateError>>;
}

impl<Substream: MutableStream> MutableStream for &mut Substream {
  type MutateError = Substream::MutateError;

  async fn mutate<const SIZE: usize, V>(
    &mut self,
    mutator: impl AsyncFnOnce(&mut [u8; SIZE]) -> V,
  ) -> Result<V, StreamMutateError<Self::MutateError>> {
    (**self).mutate(mutator).await
  }
}

pub trait ResizableStream: ReadableStream {
  type OverwriteError: UserOverwriteError;

  async fn overwrite<const SIZE: usize>(&mut self, length: u64, data: &[u8; SIZE]) -> Result<(), StreamOverwriteError<Self::OverwriteError>>;
}

impl<Substream: ResizableStream> ResizableStream for &mut Substream {
  type OverwriteError = Substream::OverwriteError;

  async fn overwrite<const SIZE: usize>(&mut self, length: u64, data: &[u8; SIZE]) -> Result<(), StreamOverwriteError<Self::OverwriteError>> {
    (**self).overwrite(length, data).await
  }
}

pub trait StaticPartitionableStream<'l, const PARTITION_SIZE: usize>: ReadableStream {
  type PartitionError: UserPartitionError;
  type Partition: ReadableStream + 'l;

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
  type PartitionDynamic: ReadableStream;

  async fn partition_dynamic(&'l mut self, size: u64) -> Result<Self::PartitionDynamic, StreamPartitionError<Self::PartitionError>>;
}

impl<'l, Substream: DynamicPartitionableStream<'l>> DynamicPartitionableStream<'l> for &mut Substream {
  type PartitionError = Substream::PartitionError;
  type PartitionDynamic = Substream::PartitionDynamic;

  async fn partition_dynamic(&'l mut self, size: u64) -> Result<Self::PartitionDynamic, StreamPartitionError<Self::PartitionError>> {
    (**self).partition_dynamic(size).await
  }
}