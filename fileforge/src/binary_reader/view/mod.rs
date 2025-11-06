use std::ops::Deref;

use crate::{
  binary_reader::{
    mutable::Mutable,
    readable::{IntoReadable, Readable},
    snapshot::BinaryReaderSnapshot,
    BinaryReader, MutableMutator,
  },
  stream::{error::stream_restore::StreamRestoreError, MutableStream, RestorableStream},
};

pub struct View<'pool, S: RestorableStream<Type = u8>, T: Readable<'pool, S>> {
  start: BinaryReaderSnapshot<'pool, S>,
  reader: BinaryReader<'pool, S>,
  value: T,
}

impl<'pool, S: RestorableStream<Type = u8>, T: Readable<'pool, S>> Deref for View<'pool, S, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.value
  }
}

impl<'pool, S: RestorableStream<Type = u8>, T: Readable<'pool, S>> IntoReadable<'pool, S> for View<'pool, S, T> {
  type Argument = T::Argument;
  type Error = T::Error;

  async fn read(mut reader: BinaryReader<'pool, S>, argument: Self::Argument) -> Result<Self, Self::Error> {
    Ok(Self {
      start: reader.snapshot(),
      value: reader.read_with(argument).await?,
      reader,
    })
  }
}

pub enum ViewMutateError<'pool, S: MutableStream<Type = u8> + RestorableStream, T: Mutable<'pool, S> + Readable<'pool, S>> {
  Restore(StreamRestoreError<S::RestoreError>),
  Mutate(<T as Mutable<'pool, S>>::Error),
}

impl<'pool, S: MutableStream<Type = u8> + RestorableStream, T: Mutable<'pool, S> + Readable<'pool, S>> View<'pool, S, T> {
  pub async fn mutate<'l>(&'l mut self) -> Result<T::Mutator<'l>, ViewMutateError<'pool, S, T>> {
    self.reader.restore(self.start.clone()).await.map_err(|e| ViewMutateError::Restore(e))?;
    self.reader.mutate::<T>().await.map_err(|e| ViewMutateError::Mutate(e))
  }
}
