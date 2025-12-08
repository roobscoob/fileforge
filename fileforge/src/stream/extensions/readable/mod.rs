pub mod byte;
pub mod filtered;
pub mod mapped;

use core::future::Future;

use crate::stream::{builtin::read_until::ReadUntil, collectable::Collectable, error::stream_read::StreamReadError, ReadableStream, SINGLE};

pub trait ReadableStreamExt: ReadableStream {
  // Transformation
  // fn map<R, Mapper: AsyncFn(Self::Type) -> R>(self, mapper: Mapper) -> MappedStream<Self, R, Mapper>;
  // fn filter<Filter: for<'a> AsyncFn(&'a Self::Type) -> bool>(self, filter: Filter) -> FilteredStream<Self, Filter>;
  //   fn filter_map<R, FilterMapper: AsyncFn(Self::Type) -> Option<R>>(self, filter_mapper: FilterMapper) -> FilteredMappedStream<Self, R, FilterMapper>;
  //   fn flatten<U>(self) -> FlattenedStream<Self, U>
  //   where
  //     Self::Type: ReadableStream<Type = U>;
  fn read_until(self, value: Self::Type) -> ReadUntil<Self>;

  // Consumption
  fn next(&mut self) -> impl Future<Output = Result<Self::Type, StreamReadError<Self::ReadError>>>
  where
    Self::Type: Copy;

  fn collect<C: Collectable<Self> + Default>(&mut self) -> impl Future<Output = Result<C, C::Error>>;
  fn collect_into<C: Collectable<Self>>(&mut self, collector: C) -> impl Future<Output = Result<C, C::Error>>;
}

impl<S: ReadableStream> ReadableStreamExt for S {
  async fn next(&mut self) -> Result<Self::Type, StreamReadError<Self::ReadError>>
  where
    Self::Type: Copy,
  {
    self.read(SINGLE).await
  }

  // fn map<R, Mapper: AsyncFn(Self::Type) -> R>(self, mapper: Mapper) -> MappedStream<Self, R, Mapper> {
  //   MappedStream { stream: self, mapper }
  // }

  // fn filter<Filter: for<'a> AsyncFn(&'a Self::Type) -> bool>(self, filter: Filter) -> FilteredStream<Self, Filter> {
  //   FilteredStream { stream: self, filter }
  // }

  fn read_until(self, value: Self::Type) -> ReadUntil<Self> {
    ReadUntil::new(self, value)
  }

  async fn collect<C: Collectable<Self> + Default>(&mut self) -> Result<C, C::Error> {
    let mut collector = C::default();

    collector.collect(self).await?;

    Ok(collector)
  }

  async fn collect_into<C: Collectable<Self>>(&mut self, mut collector: C) -> Result<C, C::Error> {
    collector.collect(self).await?;

    Ok(collector)
  }
}
