pub mod byte;
pub mod filtered;
pub mod mapped;

use core::{cmp::Ordering, future::Future};

use crate::stream::{
  builtin::read_until::ReadUntil,
  collectable::Collectable,
  error::{stream_cmp::StreamCmpError, stream_read::StreamReadError},
  ReadableStream, SINGLE,
};

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

  /// Compares the rest of this stream with the rest of `other`, element by element, the way `strcmp`
  /// compares strings: the first difference decides, and a stream that runs out first is smaller.
  /// Stops at the first difference.
  fn cmp<O: ReadableStream<Type = Self::Type>>(&mut self, other: &mut O) -> impl Future<Output = Result<Ordering, StreamCmpError<Self::ReadError, O::ReadError>>>
  where
    Self::Type: Ord + Copy;

  /// Whether the rest of this stream is the same as the rest of `other`. Stops at the first
  /// difference.
  fn eq<O: ReadableStream<Type = Self::Type>>(&mut self, other: &mut O) -> impl Future<Output = Result<bool, StreamCmpError<Self::ReadError, O::ReadError>>>
  where
    Self::Type: PartialEq + Copy;
}

/// The next element, or `None` if the stream has run out.
async fn next_or_end<S: ReadableStream>(stream: &mut S) -> Result<Option<S::Type>, S::ReadError>
where
  S::Type: Copy,
{
  match stream.read(SINGLE).await {
    Ok(item) => Ok(Some(item)),
    Err(StreamReadError::StreamExhausted(_)) => Ok(None),
    Err(StreamReadError::User(error)) => Err(error),
  }
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

  async fn cmp<O: ReadableStream<Type = Self::Type>>(&mut self, other: &mut O) -> Result<Ordering, StreamCmpError<Self::ReadError, O::ReadError>>
  where
    Self::Type: Ord + Copy,
  {
    loop {
      let first = next_or_end(self).await.map_err(StreamCmpError::First)?;
      let second = next_or_end(other).await.map_err(StreamCmpError::Second)?;

      match (first, second) {
        (None, None) => return Ok(Ordering::Equal),
        (None, Some(_)) => return Ok(Ordering::Less),
        (Some(_), None) => return Ok(Ordering::Greater),
        (Some(first), Some(second)) => match first.cmp(&second) {
          Ordering::Equal => {}
          ordering => return Ok(ordering),
        },
      }
    }
  }

  async fn eq<O: ReadableStream<Type = Self::Type>>(&mut self, other: &mut O) -> Result<bool, StreamCmpError<Self::ReadError, O::ReadError>>
  where
    Self::Type: PartialEq + Copy,
  {
    loop {
      let first = next_or_end(self).await.map_err(StreamCmpError::First)?;
      let second = next_or_end(other).await.map_err(StreamCmpError::Second)?;

      match (first, second) {
        (None, None) => return Ok(true),
        (Some(first), Some(second)) if first == second => {}
        _ => return Ok(false),
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use core::cmp::Ordering;

  use crate::{
    provider::hint::ReadHint,
    stream::{builtin::provider::ProviderStream, extensions::readable::ReadableStreamExt, ReadableStream},
  };

  fn stream(bytes: &'static [u8]) -> ProviderStream<&'static [u8]> {
    ProviderStream::new(bytes, ReadHint::new())
  }

  async fn cmp(a: &'static [u8], b: &'static [u8]) -> Ordering {
    stream(a).cmp(&mut stream(b)).await.unwrap()
  }

  #[tokio::test]
  async fn compares_like_strcmp() {
    assert_eq!(cmp(b"abc", b"abc").await, Ordering::Equal);
    assert_eq!(cmp(b"", b"").await, Ordering::Equal);
    assert_eq!(cmp(b"abc", b"abd").await, Ordering::Less);
    assert_eq!(cmp(b"abd", b"abc").await, Ordering::Greater);
    assert_eq!(cmp(b"ab", b"abc").await, Ordering::Less);
    assert_eq!(cmp(b"abc", b"ab").await, Ordering::Greater);
    assert_eq!(cmp(b"", b"a").await, Ordering::Less);
    // Bytes compare unsigned, as strcmp does.
    assert_eq!(cmp(b"\x80", b"\x7f").await, Ordering::Greater);
  }

  #[tokio::test]
  async fn equality() {
    assert!(stream(b"abc").eq(&mut stream(b"abc")).await.unwrap());
    assert!(!stream(b"abc").eq(&mut stream(b"abd")).await.unwrap());
    assert!(!stream(b"abc").eq(&mut stream(b"ab")).await.unwrap());
    assert!(!stream(b"ab").eq(&mut stream(b"abc")).await.unwrap());
  }

  #[tokio::test]
  async fn stops_at_the_first_difference() {
    let (mut a, mut b) = (stream(b"axyz"), stream(b"bxyz"));
    assert_eq!(a.cmp(&mut b).await.unwrap(), Ordering::Less);
    assert_eq!((a.offset(), b.offset()), (1, 1));
  }

  #[tokio::test]
  async fn a_null_terminated_string_against_a_name() {
    let mut name = stream(b"Model/foo.bfres\0Model/bar.bfres\0").read_until(0);
    assert!(name.eq(&mut stream(b"Model/foo.bfres")).await.unwrap());

    let mut name = stream(b"Model/foo.bfres\0").read_until(0);
    assert_eq!(name.cmp(&mut stream(b"Model/foo")).await.unwrap(), Ordering::Greater);
  }
}
