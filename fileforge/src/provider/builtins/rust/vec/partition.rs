use crate::provider::builtins::rust::vec::VecSyncResize;
use crate::provider::builtins::slice::dynamic::DynamicSliceProvider;
use crate::provider::builtins::slice::fixed::FixedSliceProvider;
use crate::provider::error::provider_mutate::ProviderMutateError;
use crate::provider::error::provider_read::ProviderReadError;
use crate::provider::error::provider_resize::ProviderResizeError;
use crate::provider::error::provider_slice::ProviderSliceError;
use crate::provider::hint::ReadHint;
use crate::provider::{error::provider_partition::ProviderPartitionError, PartitionableProvider};
use crate::provider::{MutProvider, Provider, ResizableProvider};
use core::cell::UnsafeCell;
use core::ops::Range;
use std::convert::Infallible;

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
    if at > self.len() {
      todo!()
    }

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
    let v = unsafe { &mut *self.vec.get() };

    v.resize_at_sync(self.start as u64 + offset, old_len, new_len)
  }
}
