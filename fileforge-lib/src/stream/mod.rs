pub mod builtin;
pub mod collectable;
pub mod error;

use core::{future::Future, iter::Skip, ops::Sub};

use collectable::Collectable;
use error::{
  stream_mutate::StreamMutateError, stream_overwrite::StreamOverwriteError, stream_partition::StreamPartitionError, stream_read::StreamReadError, stream_rewind::StreamRewindError,
  stream_seek::StreamSeekError, stream_skip::StreamSkipError, stream_write::StreamWriteError, user_mutate::UserMutateError, user_overwrite::UserOverwriteError, user_partition::UserPartitionError,
  user_read::UserReadError, user_rewind::UserRewindError, user_seek::UserSeekError, user_skip::UserSkipError, user_write::UserWriteError,
};

pub trait ReadableStream<'pool, const NODE_NAME_SIZE: usize>: Sized {
  type ReadError: UserReadError<'pool, NODE_NAME_SIZE>;

  fn len(&self) -> Option<u64> { return None }
  fn remaining(&self) -> Option<u64> { Some(self.len()? - self.offset()) }
  fn offset(&self) -> u64;

  fn read<const SIZE: usize, V, R: Future<Output = V>>(&mut self, reader: impl FnOnce(&[u8; SIZE]) -> R) -> impl Future<Output = Result<V, StreamReadError<NODE_NAME_SIZE, Self::ReadError>>>;

  fn collect<C: Collectable<NODE_NAME_SIZE, Self>>(&mut self, mut collector: C) -> impl Future<Output = Result<C, C::Error>> {
    async {
      collector.collect(self).await?;

      Ok(collector)
    }
  }
}

impl<'pool, const NODE_NAME_SIZE: usize, Substream: ReadableStream<'pool, NODE_NAME_SIZE>> ReadableStream<'pool, NODE_NAME_SIZE> for &mut Substream {
  type ReadError = Substream::ReadError;

  fn len(&self) -> Option<u64> { (**self).len() }
  fn remaining(&self) -> Option<u64> { (**self).remaining() }
  fn offset(&self) -> u64 { (**self).offset() }
  fn read<const SIZE: usize, V, R: Future<Output = V>>(&mut self, reader: impl FnOnce(&[u8; SIZE]) -> R) -> impl Future<Output = Result<V, StreamReadError<NODE_NAME_SIZE, Self::ReadError>>> {
    (**self).read(reader)
  }
}

pub trait SkippableStream<'pool, const NODE_NAME_SIZE: usize>: ReadableStream<'pool, NODE_NAME_SIZE> {
  type SkipError: UserSkipError<'pool, NODE_NAME_SIZE>;

  fn skip(&mut self, size: u64) -> impl Future<Output = Result<(), StreamSkipError<NODE_NAME_SIZE, Self::SkipError>>>;
}

impl<'pool, const NODE_NAME_SIZE: usize, Substream: SkippableStream<'pool, NODE_NAME_SIZE>> SkippableStream<'pool, NODE_NAME_SIZE> for &mut Substream {
  type SkipError = Substream::SkipError;

  fn skip(&mut self, size: u64) -> impl Future<Output = Result<(), StreamSkipError<NODE_NAME_SIZE, Self::SkipError>>> {
    (**self).skip(size)
  }
}

pub trait RewindableStream<'pool, const NODE_NAME_SIZE: usize>: ReadableStream<'pool, NODE_NAME_SIZE> {
  type RewindError: UserRewindError<'pool, NODE_NAME_SIZE>;

  fn rewind(&mut self, size: u64) -> impl Future<Output = Result<(), StreamRewindError<NODE_NAME_SIZE, Self::RewindError>>>;
}

impl<'pool, const NODE_NAME_SIZE: usize, Substream: RewindableStream<'pool, NODE_NAME_SIZE>> RewindableStream<'pool, NODE_NAME_SIZE> for &mut Substream {
  type RewindError = Substream::RewindError;

  fn rewind(&mut self, size: u64) -> impl Future<Output = Result<(), StreamRewindError<NODE_NAME_SIZE, Self::RewindError>>> {
    (**self).rewind(size)
  }
}

pub trait SeekableStream<'pool, const NODE_NAME_SIZE: usize>: ReadableStream<'pool, NODE_NAME_SIZE> {
  type SeekError: UserSeekError<'pool, NODE_NAME_SIZE>;

  fn seek(&mut self, offset: u64) -> impl Future<Output = Result<(), StreamSeekError<NODE_NAME_SIZE, Self::SeekError>>>;
}

impl<'pool, const NODE_NAME_SIZE: usize, Substream: SeekableStream<'pool, NODE_NAME_SIZE>> SeekableStream<'pool, NODE_NAME_SIZE> for &mut Substream {
  type SeekError = Substream::SeekError;

  fn seek(&mut self, offset: u64) -> impl Future<Output = Result<(), StreamSeekError<NODE_NAME_SIZE, Self::SeekError>>> {
    (**self).seek(offset)
  }
}

pub trait MutableStream<'pool, const NODE_NAME_SIZE: usize>: ReadableStream<'pool, NODE_NAME_SIZE> {
  type MutateError: UserMutateError<'pool, NODE_NAME_SIZE>;

  fn mutate<const SIZE: usize, V, R: Future<Output = V>>(
    &mut self,
    mutator: impl FnOnce(&mut [u8; SIZE]) -> R,
  ) -> impl Future<Output = Result<V, StreamMutateError<NODE_NAME_SIZE, Self::MutateError>>>;
}

impl<'pool, const NODE_NAME_SIZE: usize, Substream: MutableStream<'pool, NODE_NAME_SIZE>> MutableStream<'pool, NODE_NAME_SIZE> for &mut Substream {
  type MutateError = Substream::MutateError;

  fn mutate<const SIZE: usize, V, R: Future<Output = V>>(
    &mut self,
    mutator: impl FnOnce(&mut [u8; SIZE]) -> R,
  ) -> impl Future<Output = Result<V, StreamMutateError<NODE_NAME_SIZE, Self::MutateError>>> {
    (**self).mutate(mutator)
  }
}

pub trait ResizableStream<'pool, const NODE_NAME_SIZE: usize>: ReadableStream<'pool, NODE_NAME_SIZE> {
  type OverwriteError: UserOverwriteError<'pool, NODE_NAME_SIZE>;

  fn overwrite<const SIZE: usize>(&mut self, length: u64, data: &[u8; SIZE]) -> impl Future<Output = Result<(), StreamOverwriteError<NODE_NAME_SIZE, Self::OverwriteError>>>;
}

impl<'pool, const NODE_NAME_SIZE: usize, Substream: ResizableStream<'pool, NODE_NAME_SIZE>> ResizableStream<'pool, NODE_NAME_SIZE> for &mut Substream {
  type OverwriteError = Substream::OverwriteError;

  fn overwrite<const SIZE: usize>(&mut self, length: u64, data: &[u8; SIZE]) -> impl Future<Output = Result<(), StreamOverwriteError<NODE_NAME_SIZE, Self::OverwriteError>>> {
    (**self).overwrite(length, data)
  }
}

pub trait StaticPartitionableStream<'pool, 'l, const NODE_NAME_SIZE: usize, const PARTITION_SIZE: usize>: ReadableStream<NODE_NAME_SIZE> {
  type PartitionError: UserPartitionError<'pool, NODE_NAME_SIZE>;
  type Partition: ReadableStream<NODE_NAME_SIZE> + 'l;

  fn partition(&'l mut self) -> impl Future<Output = Result<Self::Partition, StreamPartitionError<NODE_NAME_SIZE, Self::PartitionError>>>;
}

impl<'pool, 'l, const NODE_NAME_SIZE: usize, const PARTITION_SIZE: usize, Substream: StaticPartitionableStream<'pool, 'l, NODE_NAME_SIZE, PARTITION_SIZE>> StaticPartitionableStream<'pool, 'l, NODE_NAME_SIZE, PARTITION_SIZE> for &mut Substream {
  type PartitionError = Substream::PartitionError;
  type Partition = Substream::Partition;

  fn partition(&'l mut self) -> impl Future<Output = Result<Self::Partition, StreamPartitionError<NODE_NAME_SIZE, Self::PartitionError>>> {
    (**self).partition()
  }
}

pub trait DynamicPartitionableStream<'pool, 'l, const NODE_NAME_SIZE: usize>: ReadableStream<'pool, NODE_NAME_SIZE> {
  type PartitionError: UserPartitionError<'pool, NODE_NAME_SIZE>;
  type PartitionDynamic: ReadableStream<NODE_NAME_SIZE>;

  fn partition_dynamic(&'l mut self, size: u64) -> impl Future<Output = Result<Self::PartitionDynamic, StreamPartitionError<NODE_NAME_SIZE, Self::PartitionError>>>;
}

impl<'pool, 'l, const NODE_NAME_SIZE: usize, Substream: DynamicPartitionableStream<'pool, 'l, NODE_NAME_SIZE>> DynamicPartitionableStream<'pool, 'l, NODE_NAME_SIZE> for &mut Substream {
  type PartitionError = Substream::PartitionError;
  type PartitionDynamic = Substream::PartitionDynamic;

  fn partition_dynamic(&'l mut self, size: u64) -> impl Future<Output = Result<Self::PartitionDynamic, StreamPartitionError<NODE_NAME_SIZE, Self::PartitionError>>> {
    (**self).partition_dynamic(size)
  }
}

pub trait WritableStream<'pool, const NODE_NAME_SIZE: usize> {
  type WriteError: UserWriteError<'pool, NODE_NAME_SIZE>;

  fn write<const SIZE: usize, V, R: Future<Output = V>>(&mut self, writer: impl FnOnce(&mut [u8; SIZE]) -> R) -> impl Future<Output = Result<V, StreamWriteError<NODE_NAME_SIZE, Self::WriteError>>>;
}

impl<'pool, const NODE_NAME_SIZE: usize, Substream: WritableStream<'pool, NODE_NAME_SIZE>> WritableStream<'pool, NODE_NAME_SIZE> for &mut Substream {
  type WriteError = Substream::WriteError;

  fn write<const SIZE: usize, V, R: Future<Output = V>>(&mut self, writer: impl FnOnce(&mut [u8; SIZE]) -> R) -> impl Future<Output = Result<V, StreamWriteError<NODE_NAME_SIZE, Self::WriteError>>> {
    (**self).write(writer)
  }
}
