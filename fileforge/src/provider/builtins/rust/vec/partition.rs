use crate::provider::builtins::rust::vec::VecSyncResize;
use crate::provider::builtins::slice::dynamic::DynamicSliceProvider;
use crate::provider::builtins::slice::fixed::FixedSliceProvider;
use crate::provider::error::out_of_bounds::OutOfBoundsError;
use crate::provider::error::provider_mutate::ProviderMutateError;
use crate::provider::error::provider_read::ProviderReadError;
use crate::provider::error::provider_resize::ProviderResizeError;
use crate::provider::error::provider_slice::ProviderSliceError;
use crate::provider::hint::ReadHint;
use crate::provider::{error::provider_partition::ProviderPartitionError, PartitionableProvider};
use crate::provider::{MutProvider, Provider, ResizableProvider};
use core::cell::UnsafeCell;
use core::ops::Range;
use core::convert::Infallible;

pub struct Head<'a, T> {
  vec: &'a UnsafeCell<alloc::vec::Vec<T>>,
  range: Range<usize>,
}

pub struct Tail<'a, T> {
  vec: &'a UnsafeCell<alloc::vec::Vec<T>>,
  start: usize,
}

impl<'a, T> PartitionableProvider for &'a mut alloc::vec::Vec<T>
where
  T: Copy,
{
  type PartitionError = Infallible;

  type PartitionLeftProvider = Head<'a, T>;
  type PartitionRightProvider = Tail<'a, T>;

  fn partition(self, at: u64) -> Result<(Self::PartitionLeftProvider, Self::PartitionRightProvider), ProviderPartitionError<Self::PartitionError>> {
    // The left half is the region [0, at). Describing it as a region (rather than a seek to `at`)
    // gives the error a length, which the stream layer needs to report it.
    OutOfBoundsError::assert(Provider::len(&*self), 0, Some(at))?;

    let vec = &*UnsafeCell::from_mut(self);

    let head = Head { vec, range: 0..at as usize };
    let tail = Tail { vec, start: at as usize };

    Ok((head, tail))
  }
}

impl<'a, T> Provider for Head<'a, T>
where
  T: Copy,
{
  type Type = T;

  type StaticSliceProvider<'l, const SIZE: usize>
    = FixedSliceProvider<SIZE, &'l Self>
  where
    Self: 'l;

  type DynamicSliceProvider<'l>
    = DynamicSliceProvider<&'l Self>
  where
    Self: 'l;

  type ReadError = Infallible;
  type SliceError = Infallible;

  fn len(&self) -> u64 {
    self.range.end as u64 - self.range.start as u64
  }

  async fn read<const SIZE: usize, V>(&self, offset: u64, _: ReadHint, reader: impl for<'v> AsyncFnOnce(&'v [Self::Type; SIZE]) -> V) -> Result<V, ProviderReadError<Self::ReadError>> {
    OutOfBoundsError::assert(self.len(), offset, Some(SIZE as u64))?;

    let v: &[T; SIZE] = unsafe { &*self.vec.get() }.as_slice()[self.range.clone()][offset as usize..offset as usize + SIZE].try_into().unwrap();
    let v: [T; SIZE] = *v;

    Ok((reader)(&v).await)
  }

  fn slice<'l, const SIZE: usize>(&'l self, start: u64) -> Result<Self::StaticSliceProvider<'l, SIZE>, ProviderSliceError<Self::SliceError>> {
    Ok(FixedSliceProvider::new(start, self)?)
  }

  fn slice_dynamic<'l>(&'l self, start: u64, size: Option<u64>) -> Result<Self::DynamicSliceProvider<'l>, ProviderSliceError<Self::SliceError>> {
    Ok(DynamicSliceProvider::new(start, size, self)?)
  }
}

impl<'a, T> Provider for Tail<'a, T>
where
  T: Copy,
{
  type Type = T;

  type StaticSliceProvider<'l, const SIZE: usize>
    = FixedSliceProvider<SIZE, &'l Self>
  where
    Self: 'l;

  type DynamicSliceProvider<'l>
    = DynamicSliceProvider<&'l Self>
  where
    Self: 'l;

  type ReadError = Infallible;
  type SliceError = Infallible;

  fn len(&self) -> u64 {
    unsafe { &*self.vec.get() }.len() as u64 - self.start as u64
  }

  async fn read<const SIZE: usize, V>(&self, offset: u64, _: ReadHint, reader: impl for<'v> AsyncFnOnce(&'v [Self::Type; SIZE]) -> V) -> Result<V, ProviderReadError<Self::ReadError>> {
    OutOfBoundsError::assert(self.len(), offset, Some(SIZE as u64))?;

    let v: &[T; SIZE] = unsafe { &*self.vec.get() }.as_slice()[self.start..][offset as usize..offset as usize + SIZE].try_into().unwrap();
    let v: [T; SIZE] = *v;

    Ok((reader)(&v).await)
  }

  fn slice<'l, const SIZE: usize>(&'l self, start: u64) -> Result<Self::StaticSliceProvider<'l, SIZE>, ProviderSliceError<Self::SliceError>> {
    Ok(FixedSliceProvider::new(start, self)?)
  }

  fn slice_dynamic<'l>(&'l self, start: u64, size: Option<u64>) -> Result<Self::DynamicSliceProvider<'l>, ProviderSliceError<Self::SliceError>> {
    Ok(DynamicSliceProvider::new(start, size, self)?)
  }
}

impl<'a, T> MutProvider for Head<'a, T>
where
  T: Copy,
{
  type MutateError = Infallible;

  type StaticMutSliceProvider<'l, const SIZE: usize>
    = FixedSliceProvider<SIZE, &'l mut Self>
  where
    Self: 'l;

  type DynamicMutSliceProvider<'l>
    = DynamicSliceProvider<&'l mut Self>
  where
    Self: 'l;

  async fn mutate<const SIZE: usize, V>(&mut self, offset: u64, writer: impl for<'v> AsyncFnOnce(&'v mut [Self::Type; SIZE]) -> V) -> Result<V, ProviderMutateError<Self::MutateError>> {
    OutOfBoundsError::assert(self.len(), offset, Some(SIZE as u64))?;

    let v: &[T; SIZE] = unsafe { &*self.vec.get() }.as_slice()[self.range.clone()][offset as usize..offset as usize + SIZE].try_into().unwrap();
    let mut v: [T; SIZE] = *v;

    let result = (writer)(&mut v).await;

    let x: &mut [T] = unsafe { &mut *self.vec.get() }.as_mut_slice();
    let v2: &mut [T; SIZE] = (&mut (&mut x[self.range.clone()])[offset as usize..offset as usize + SIZE]).try_into().unwrap();

    *v2 = v;

    Ok(result)
  }

  fn mut_slice<'l, const SIZE: usize>(&'l mut self, start: u64) -> Result<Self::StaticMutSliceProvider<'l, SIZE>, ProviderSliceError<Self::SliceError>> {
    Ok(FixedSliceProvider::new(start, self)?)
  }

  fn mut_slice_dynamic<'l>(&'l mut self, start: u64, size: Option<u64>) -> Result<Self::DynamicMutSliceProvider<'l>, ProviderSliceError<Self::SliceError>> {
    Ok(DynamicSliceProvider::new(start, size, self)?)
  }
}

impl<'a, T> MutProvider for Tail<'a, T>
where
  T: Copy,
{
  type MutateError = Infallible;

  type StaticMutSliceProvider<'l, const SIZE: usize>
    = FixedSliceProvider<SIZE, &'l mut Self>
  where
    Self: 'l;

  type DynamicMutSliceProvider<'l>
    = DynamicSliceProvider<&'l mut Self>
  where
    Self: 'l;

  async fn mutate<const SIZE: usize, V>(&mut self, offset: u64, writer: impl for<'v> AsyncFnOnce(&'v mut [Self::Type; SIZE]) -> V) -> Result<V, ProviderMutateError<Self::MutateError>> {
    OutOfBoundsError::assert(self.len(), offset, Some(SIZE as u64))?;

    let v: &[T; SIZE] = unsafe { &*self.vec.get() }.as_slice()[self.start..][offset as usize..offset as usize + SIZE].try_into().unwrap();
    let mut v: [T; SIZE] = *v;

    let result = (writer)(&mut v).await;

    let x: &mut [T] = unsafe { &mut *self.vec.get() }.as_mut_slice();
    let v2: &mut [T; SIZE] = (&mut (&mut x[self.start..])[offset as usize..offset as usize + SIZE]).try_into().unwrap();

    *v2 = v;

    Ok(result)
  }

  fn mut_slice<'l, const SIZE: usize>(&'l mut self, start: u64) -> Result<Self::StaticMutSliceProvider<'l, SIZE>, ProviderSliceError<Self::SliceError>> {
    Ok(FixedSliceProvider::new(start, self)?)
  }

  fn mut_slice_dynamic<'l>(&'l mut self, start: u64, size: Option<u64>) -> Result<Self::DynamicMutSliceProvider<'l>, ProviderSliceError<Self::SliceError>> {
    Ok(DynamicSliceProvider::new(start, size, self)?)
  }
}

impl<'a, T> ResizableProvider for Tail<'a, T>
where
  T: Copy,
  T: Default,
{
  type ResizeError = Infallible;

  async fn resize_at(&mut self, offset: u64, old_len: u64, new_len: u64) -> Result<(), ProviderResizeError<Self::ResizeError>> {
    // Check against this half, so the error describes it (not the whole Vec), and so
    // `start + offset` below can't overflow.
    OutOfBoundsError::assert(self.len(), offset, Some(old_len))?;

    let v = unsafe { &mut *self.vec.get() };

    v.resize_at_sync(self.start as u64 + offset, old_len, new_len)
  }
}

#[cfg(test)]
mod tests {
  use std::vec;

  use crate::{
    provider::{
      error::{provider_mutate::ProviderMutateError, provider_partition::ProviderPartitionError, provider_read::ProviderReadError, provider_resize::ProviderResizeError},
      hint::ReadHint,
      MutProvider, PartitionableProvider, Provider, ResizableProvider,
    },
    stream::{
      builtin::provider::ProviderStream,
      error::{stream_exhausted::StreamExhaustedError, stream_partition::StreamPartitionError},
      DynamicPartitionableStream, ReadableStream,
    },
  };

  #[tokio::test]
  async fn partition_splits_in_the_middle() {
    let mut v = vec![1u8, 2, 3, 4, 5];
    let Ok((head, tail)) = PartitionableProvider::partition(&mut v, 2) else { panic!("partition should succeed") };

    assert_eq!(head.len(), 2);
    assert_eq!(tail.len(), 3);

    let got = tail.read::<3, _>(0, ReadHint::new(), async move |c| [c[0], c[1], c[2]]).await.ok();
    assert_eq!(got, Some([3, 4, 5]));
  }

  #[test]
  fn partition_at_the_end_gives_an_empty_tail() {
    let mut v = vec![1u8, 2, 3];
    let Ok((head, tail)) = PartitionableProvider::partition(&mut v, 3) else { panic!("partition at the end should succeed") };

    assert_eq!(head.len(), 3);
    assert_eq!(tail.len(), 0);
  }

  #[test]
  fn partition_past_the_end_is_out_of_bounds() {
    let mut v = vec![1u8, 2, 3];

    match PartitionableProvider::partition(&mut v, 4) {
      Err(ProviderPartitionError::OutOfBounds(oob)) => {
        assert_eq!((oob.read_offset, oob.read_length, oob.provider_size), (0, Some(4), 3));

        // The stream layer unwraps this conversion, so it must succeed.
        let exhausted = Option::<StreamExhaustedError>::from(oob).expect("the error must have a length");
        assert_eq!(exhausted.read_offset + exhausted.read_length, 4);
      }
      Err(other) => panic!("expected OutOfBounds, got {other:?}"),
      Ok(_) => panic!("partition past the end should fail"),
    }
  }

  #[tokio::test]
  async fn reading_past_the_end_of_either_half_is_out_of_bounds() {
    let mut v = vec![1u8, 2, 3, 4, 5];
    let Ok((head, tail)) = PartitionableProvider::partition(&mut v, 2) else { panic!("partition should succeed") };

    let head_read = head.read::<2, _>(1, ReadHint::new(), async move |c| c[0]).await;
    assert!(matches!(head_read, Err(ProviderReadError::OutOfBounds(_))));

    let tail_read = tail.read::<2, _>(2, ReadHint::new(), async move |c| c[0]).await;
    assert!(matches!(tail_read, Err(ProviderReadError::OutOfBounds(_))));
  }

  #[tokio::test]
  async fn mutating_past_the_end_of_either_half_is_out_of_bounds() {
    let mut v = vec![1u8, 2, 3, 4, 5];
    let Ok((mut head, mut tail)) = PartitionableProvider::partition(&mut v, 2) else { panic!("partition should succeed") };

    let head_mutate = head.mutate::<2, _>(1, async move |c| c[0] = 0).await;
    assert!(matches!(head_mutate, Err(ProviderMutateError::OutOfBounds(_))));

    let tail_mutate = tail.mutate::<2, _>(2, async move |c| c[0] = 0).await;
    assert!(matches!(tail_mutate, Err(ProviderMutateError::OutOfBounds(_))));

    assert_eq!(v, vec![1, 2, 3, 4, 5], "nothing should have been written");
  }

  #[tokio::test]
  async fn resizing_past_the_end_of_the_tail_is_out_of_bounds() {
    let mut v = vec![1u8, 2, 3, 4, 5];
    let Ok((_, mut tail)) = PartitionableProvider::partition(&mut v, 2) else { panic!("partition should succeed") };

    // The tail is 3 long, so replacing 2 items at offset 2 runs past it.
    match tail.resize_at(2, 2, 4).await {
      Err(ProviderResizeError::OutOfBounds(oob)) => assert_eq!(oob.provider_size, 3, "the error should describe the tail, not the whole Vec"),
      other => panic!("expected OutOfBounds, got {other:?}"),
    }

    // A huge offset must be an error, not an overflow.
    assert!(matches!(tail.resize_at(u64::MAX, 1, 1).await, Err(ProviderResizeError::OutOfBounds(_))));
  }

  #[tokio::test]
  async fn stream_partition_past_the_end_describes_the_read() {
    let mut v = vec![1u8, 2, 3, 4, 5];
    let mut stream = ProviderStream::new(&mut v, ReadHint::new());
    assert!(stream.skip(1).await.is_ok());

    match stream.partition_dynamic(10).await {
      Err(StreamPartitionError::StreamExhausted(exhausted)) => {
        assert_eq!((exhausted.read_offset, exhausted.read_length, exhausted.stream_length), (1, 10, 5));
      }
      Err(StreamPartitionError::User(never)) => match never {},
      Ok(_) => panic!("partition past the end should fail"),
    }
  }

  #[tokio::test]
  async fn stream_partition_that_overflows_is_an_error() {
    let mut v = vec![1u8, 2, 3, 4, 5];
    let mut stream = ProviderStream::new(&mut v, ReadHint::new());
    assert!(stream.skip(1).await.is_ok());

    // offset 1 + u64::MAX overflows; this used to panic (or, in release builds, wrap to a wrong split).
    match stream.partition_dynamic(u64::MAX).await {
      Err(StreamPartitionError::StreamExhausted(exhausted)) => {
        assert_eq!((exhausted.read_offset, exhausted.read_length), (1, u64::MAX));
      }
      Err(StreamPartitionError::User(never)) => match never {},
      Ok(_) => panic!("an overflowing partition should fail"),
    }
  }
}
