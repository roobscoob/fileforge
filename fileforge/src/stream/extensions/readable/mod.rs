pub mod filtered;
pub mod mapped;

use core::future::Future;

use crate::stream::{
  error::stream_read::StreamReadError,
  extensions::readable::{filtered::FilteredStream, mapped::MappedStream},
  ReadableStream, SINGLE,
};

pub trait ReadableStreamExt: ReadableStream {
  // Transformation
  fn map<R, Mapper: AsyncFn(Self::Type) -> R>(self, mapper: Mapper) -> MappedStream<Self, R, Mapper>;
  fn filter<Filter: for<'a> AsyncFn(&'a Self::Type) -> bool>(self, filter: Filter) -> FilteredStream<Self, Filter>;
  //   fn filter_map<R, FilterMapper: AsyncFn(Self::Type) -> Option<R>>(self, filter_mapper: FilterMapper) -> FilteredMappedStream<Self, R, FilterMapper>;
  //   fn flatten<U>(self) -> FlattenedStream<Self, U>
  //   where
  //     Self::Type: ReadableStream<Type = U>;

  // Consumption
  fn next(&mut self) -> impl Future<Output = Result<Self::Type, StreamReadError<Self::ReadError>>>
  where
    Self::Type: Copy;
}

impl<S: ReadableStream> ReadableStreamExt for S {
  async fn next(&mut self) -> Result<Self::Type, StreamReadError<Self::ReadError>>
  where
    Self::Type: Copy,
  {
    self.read(SINGLE).await
  }

  fn map<R, Mapper: AsyncFn(Self::Type) -> R>(self, mapper: Mapper) -> MappedStream<Self, R, Mapper> {
    MappedStream { stream: self, mapper }
  }

  fn filter<Filter: for<'a> AsyncFn(&'a Self::Type) -> bool>(self, filter: Filter) -> FilteredStream<Self, Filter> {
    FilteredStream { stream: self, filter }
  }
}
