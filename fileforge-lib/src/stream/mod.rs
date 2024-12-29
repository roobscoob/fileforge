pub mod builtin;
pub mod collectable;
pub mod error;

use core::future::Future;

use collectable::Collectable;
use error::{
  stream_mutate::StreamMutateError, stream_overwrite::StreamOverwriteError, stream_read::StreamReadError, stream_rewind::StreamRewindError, stream_seek::StreamSeekError, stream_skip::StreamSkipError,
  stream_write::StreamWriteError, user_mutate::UserMutateError, user_overwrite::UserOverwriteError, user_read::UserReadError, user_rewind::UserRewindError, user_seek::UserSeekError,
  user_skip::UserSkipError, user_write::UserWriteError,
};

pub trait ReadableStream<T, const NODE_NAME_SIZE: usize>: Sized {
  type ReadError: UserReadError<NODE_NAME_SIZE>;

  fn len(&self) -> Option<u64> { return None }

  fn read<const SIZE: usize, V, R: Future<Output = V>>(&mut self, reader: impl FnOnce(&[T; SIZE]) -> R) -> impl Future<Output = Result<V, StreamReadError<NODE_NAME_SIZE, Self::ReadError>>>;

  fn collect<C: Collectable<NODE_NAME_SIZE, T, Self>>(&mut self, mut collector: C) -> impl Future<Output = Result<C, C::Error>> {
    async {
      collector.collect(self).await?;

      Ok(collector)
    }
  }
}

pub trait SkippableStream<T, const NODE_NAME_SIZE: usize>: ReadableStream<T, NODE_NAME_SIZE> {
  type SkipError: UserSkipError<NODE_NAME_SIZE>;

  fn skip(&mut self, size: u64) -> impl Future<Output = Result<(), StreamSkipError<NODE_NAME_SIZE, Self::SkipError>>>;
}

pub trait RewindableStream<T, const NODE_NAME_SIZE: usize>: ReadableStream<T, NODE_NAME_SIZE> {
  type RewindError: UserRewindError<NODE_NAME_SIZE>;

  fn rewind(&mut self, size: u64) -> impl Future<Output = Result<(), StreamRewindError<NODE_NAME_SIZE, Self::RewindError>>>;
}

pub trait SeekableStream<T, const NODE_NAME_SIZE: usize>: ReadableStream<T, NODE_NAME_SIZE> {
  type SeekError: UserSeekError<NODE_NAME_SIZE>;

  fn seek(&mut self, offset: u64) -> impl Future<Output = Result<(), StreamSeekError<NODE_NAME_SIZE, Self::SeekError>>>;
}

pub trait MutableStream<T, const NODE_NAME_SIZE: usize>: ReadableStream<T, NODE_NAME_SIZE> {
  type MutateError: UserMutateError<NODE_NAME_SIZE>;

  fn mutate<const SIZE: usize, V, R: Future<Output = V>>(&mut self, reader: impl FnOnce(&[T; SIZE]) -> R) -> impl Future<Output = Result<V, StreamMutateError<NODE_NAME_SIZE, Self::MutateError>>>;
}

pub trait ResizableStream<T, const NODE_NAME_SIZE: usize>: ReadableStream<T, NODE_NAME_SIZE> {
  type OverwriteError: UserOverwriteError<NODE_NAME_SIZE>;

  fn overwrite(&mut self, length: u64, data: &[u8]) -> impl Future<Output = Result<(), StreamOverwriteError<NODE_NAME_SIZE, Self::OverwriteError>>>;
}

pub trait WritableStream<T, const NODE_NAME_SIZE: usize> {
  type WriteError: UserWriteError<NODE_NAME_SIZE>;

  fn write<const SIZE: usize, V, R: Future<Output = V>>(&mut self, writer: impl FnOnce(&mut [T; SIZE]) -> R) -> impl Future<Output = Result<V, StreamWriteError<NODE_NAME_SIZE, Self::WriteError>>>;
}
