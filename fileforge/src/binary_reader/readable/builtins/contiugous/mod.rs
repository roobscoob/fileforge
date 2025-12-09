use core::{convert::Infallible, marker::PhantomData};

use crate::{
  binary_reader::{
    self,
    readable::{builtins::array::ArrayReadError, IntoReadable, Readable, RefReadable},
    BinaryReader,
  },
  error::FileforgeError,
  stream::{self, ReadableStream},
};

pub struct Contiguous<'pool, S: ReadableStream<Type = u8>, T: Readable<'pool, S>, Gen: FnMut(u64) -> T::Argument> {
  reader: BinaryReader<'pool, S>,
  index: u64,
  generator: Gen,
  _phantom: PhantomData<fn() -> T>,
}

extern crate std;

impl<'pool, S: ReadableStream<Type = u8>, T: Readable<'pool, S>, Gen: FnMut(u64) -> T::Argument> Contiguous<'pool, S, T, Gen> {
  pub async fn finish(mut self, length: u64) -> Result<(), stream::StreamSkipError<ContiguousSkipError<'pool, <S as ReadableStream>::SkipError, <T as Readable<'pool, S>>::Error>>> {
    self.skip(length.saturating_sub(self.index)).await
  }
}

impl<'s, 'pool: 's, S: ReadableStream<Type = u8>, T: Readable<'pool, &'s mut S> + 's, Gen: 's + FnMut(u64) -> T::Argument> RefReadable<'s, 'pool, S> for Contiguous<'pool, &'s mut S, T, Gen> {
  type Error = Infallible;

  type Argument = Gen;

  async fn read_ref(reader: &'s mut BinaryReader<'pool, S>, generator: Self::Argument) -> Result<Self, Self::Error> {
    Ok(Self {
      reader: reader.borrow_fork(),
      index: 0,
      generator,
      _phantom: PhantomData,
    })
  }
}

impl<'pool, S: ReadableStream<Type = u8>, T: Readable<'pool, S>, Gen: FnMut(u64) -> T::Argument> IntoReadable<'pool, S> for Contiguous<'pool, S, T, Gen> {
  type Error = Infallible;

  type Argument = Gen;

  async fn read(reader: BinaryReader<'pool, S>, generator: Self::Argument) -> Result<Self, Self::Error> {
    Ok(Self {
      reader,
      index: 0,
      generator,
      _phantom: PhantomData,
    })
  }
}

pub enum ContiguousSkipError<'pool, S: stream::UserSkipError, R: FileforgeError> {
  Overflowed, // todo: item size + size
  Read { index: u64, read_error: R },
  Stream(binary_reader::SkipError<'pool, S>),
}
impl<'pool, S: stream::UserSkipError, R: FileforgeError> FileforgeError for ContiguousSkipError<'pool, S, R> {
  fn render_into_report<P: crate::diagnostic::pool::DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(
    &self,
    provider: P,
    callback: impl for<'tag, 'b> FnOnce(crate::error::report::Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
  ) {
    todo!()
  }
}
impl<'pool, S: stream::UserSkipError, E: FileforgeError> stream::UserSkipError for ContiguousSkipError<'pool, S, E> {}

impl<'pool, S: ReadableStream<Type = u8>, T: Readable<'pool, S>, Gen: FnMut(u64) -> T::Argument> ReadableStream for Contiguous<'pool, S, T, Gen> {
  type Type = T;

  type ReadError = ArrayReadError<T::Error>;

  type SkipError = ContiguousSkipError<'pool, S::SkipError, T::Error>;

  fn offset(&self) -> u64 {
    self.index as u64
  }

  async fn read<const SIZE: usize, V>(&mut self, reader: impl AsyncFnOnce(&[Self::Type; SIZE]) -> V) -> Result<V, stream::StreamReadError<Self::ReadError>> {
    let arguments = core::array::from_fn(|index| (self.generator)(self.index + index as u64));
    let actual = self.reader.read_with::<[T; SIZE]>(arguments).await.map_err(stream::StreamReadError::User)?;

    self.index += SIZE as u64;

    Ok(reader(&actual).await)
  }

  async fn skip(&mut self, size: u64) -> Result<(), stream::StreamSkipError<Self::SkipError>> {
    if let Some(item_size) = T::SIZE {
      let total_size = size.checked_mul(item_size).ok_or(stream::StreamSkipError::User(ContiguousSkipError::Overflowed))?;
      self.index += size;
      self.reader.skip(total_size).await.map_err(ContiguousSkipError::Stream).map_err(stream::StreamSkipError::User)
    } else {
      // ensure that we can read at `size` items
      self.index.checked_add(size).ok_or(stream::StreamSkipError::User(ContiguousSkipError::Overflowed))?;

      for i in 0..size {
        self
          .reader
          .read_with::<T>((self.generator)(self.index + i))
          .await
          .map_err(|error| ContiguousSkipError::Read {
            index: self.index + i,
            read_error: error,
          })
          .map_err(stream::StreamSkipError::User)?;
      }

      self.index += size;
      Ok(())
    }
  }
}
