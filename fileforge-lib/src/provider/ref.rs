use core::future::Future;

use super::{
  error::{provider_read::ProviderReadError, provider_slice::ProviderSliceError},
  hint::ReadHint,
  MutProvider, Provider, ResizableProvider,
};

impl<const NODE_NAME_SIZE: usize, P: Provider<NODE_NAME_SIZE>> Provider<NODE_NAME_SIZE> for &P {
  type StaticSliceProvider<'l, const SIZE: usize>
    = P::StaticSliceProvider<'l, SIZE>
  where
    Self: 'l;

  type DynamicSliceProvider<'l>
    = P::DynamicSliceProvider<'l>
  where
    Self: 'l;

  type SliceError = P::SliceError;
  type ReadError = P::ReadError;

  fn len(&self) -> u64 { (**self).len() }

  fn slice<'l, const SIZE: usize>(&'l self, start: u64) -> Result<Self::StaticSliceProvider<'l, SIZE>, ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>> { (**self).slice::<SIZE>(start) }

  fn slice_dynamic<'l>(&'l self, start: u64, size: Option<u64>) -> Result<Self::DynamicSliceProvider<'l>, ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>> { (**self).slice_dynamic(start, size) }

  fn read<const SIZE: usize, V, R: Future<Output = V>>(
    &self,
    offset: u64,
    hint: ReadHint,
    reader: impl FnOnce(&[u8; SIZE]) -> R,
  ) -> impl Future<Output = Result<V, ProviderReadError<NODE_NAME_SIZE, Self::ReadError>>> {
    (**self).read(offset, hint, reader)
  }
}

impl<const NODE_NAME_SIZE: usize, P: Provider<NODE_NAME_SIZE>> Provider<NODE_NAME_SIZE> for &mut P {
  type StaticSliceProvider<'l, const SIZE: usize>
    = P::StaticSliceProvider<'l, SIZE>
  where
    Self: 'l;

  type DynamicSliceProvider<'l>
    = P::DynamicSliceProvider<'l>
  where
    Self: 'l;

  type SliceError = P::SliceError;
  type ReadError = P::ReadError;

  fn len(&self) -> u64 { (**self).len() }

  fn slice<'l, const SIZE: usize>(&'l self, start: u64) -> Result<Self::StaticSliceProvider<'l, SIZE>, ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>> { (**self).slice::<SIZE>(start) }

  fn slice_dynamic<'l>(&'l self, start: u64, size: Option<u64>) -> Result<Self::DynamicSliceProvider<'l>, ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>> { (**self).slice_dynamic(start, size) }

  fn read<const SIZE: usize, V, R: Future<Output = V>>(
    &self,
    offset: u64,
    hint: ReadHint,
    reader: impl FnOnce(&[u8; SIZE]) -> R,
  ) -> impl Future<Output = Result<V, ProviderReadError<NODE_NAME_SIZE, Self::ReadError>>> {
    (**self).read(offset, hint, reader)
  }
}

impl<const NODE_NAME_SIZE: usize, P: MutProvider<NODE_NAME_SIZE>> MutProvider<NODE_NAME_SIZE> for &mut P {
  type MutateError = P::MutateError;

  type StaticMutSliceProvider<'l, const SIZE: usize>
    = P::StaticMutSliceProvider<'l, SIZE>
  where
    Self: 'l;

  type DynamicMutSliceProvider<'l>
    = P::DynamicMutSliceProvider<'l>
  where
    Self: 'l;

  fn mutate<const SIZE: usize, V, R: Future<Output = V>>(
    &mut self,
    offset: u64,
    writer: impl FnOnce(&mut [u8; SIZE]) -> R,
  ) -> impl Future<Output = Result<V, crate::provider::error::provider_mutate::ProviderMutateError<NODE_NAME_SIZE, Self::MutateError>>> {
    (**self).mutate(offset, writer)
  }

  fn mut_slice<'l, const SIZE: usize>(&'l mut self, start: u64) -> Result<Self::StaticMutSliceProvider<'l, SIZE>, ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>> {
    (**self).mut_slice::<SIZE>(start)
  }

  fn mut_slice_dynamic<'l>(&'l mut self, start: u64, size: Option<u64>) -> Result<Self::DynamicMutSliceProvider<'l>, ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>> {
    (**self).mut_slice_dynamic(start, size)
  }
}

impl<const NODE_NAME_SIZE: usize, P: ResizableProvider<NODE_NAME_SIZE>> ResizableProvider<NODE_NAME_SIZE> for &mut P {
  type ResizeError = P::ResizeError;

  async fn resize_at(&mut self, offset: u64, old_len: u64, new_len: u64) -> Result<(), super::error::provider_resize::ProviderResizeError<NODE_NAME_SIZE, Self::ResizeError>> {
    (**self).resize_at(offset, old_len, new_len).await
  }
}
