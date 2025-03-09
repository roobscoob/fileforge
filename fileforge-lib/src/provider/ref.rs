use core::future::Future;

use super::{
  error::{provider_read::ProviderReadError, provider_slice::ProviderSliceError},
  hint::ReadHint,
  MutProvider, Provider, ResizableProvider,
};

impl<P: Provider> Provider for &P {
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

  fn slice<'l, const SIZE: usize>(&'l self, start: u64) -> Result<Self::StaticSliceProvider<'l, SIZE>, ProviderSliceError<Self::SliceError>> { (**self).slice::<SIZE>(start) }

  fn slice_dynamic<'l>(&'l self, start: u64, size: Option<u64>) -> Result<Self::DynamicSliceProvider<'l>, ProviderSliceError<Self::SliceError>> { (**self).slice_dynamic(start, size) }

  async fn read<const SIZE: usize, V>(
    &self,
    offset: u64,
    hint: ReadHint,
    reader: impl AsyncFnOnce(&[u8; SIZE]) -> V,
  ) -> Result<V, ProviderReadError<Self::ReadError>> {
    (**self).read(offset, hint, reader).await
  }
}

impl<P: Provider> Provider for &mut P {
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

  fn slice<'l, const SIZE: usize>(&'l self, start: u64) -> Result<Self::StaticSliceProvider<'l, SIZE>, ProviderSliceError<Self::SliceError>> { (**self).slice::<SIZE>(start) }

  fn slice_dynamic<'l>(&'l self, start: u64, size: Option<u64>) -> Result<Self::DynamicSliceProvider<'l>, ProviderSliceError<Self::SliceError>> { (**self).slice_dynamic(start, size) }

  async fn read<const SIZE: usize, V>(
    &self,
    offset: u64,
    hint: ReadHint,
    reader: impl AsyncFnOnce(&[u8; SIZE]) -> V,
  ) -> Result<V, ProviderReadError<Self::ReadError>> {
    (**self).read(offset, hint, reader).await
  }
}

impl<P: MutProvider> MutProvider for &mut P {
  type MutateError = P::MutateError;

  type StaticMutSliceProvider<'l, const SIZE: usize>
    = P::StaticMutSliceProvider<'l, SIZE>
  where
    Self: 'l;

  type DynamicMutSliceProvider<'l>
    = P::DynamicMutSliceProvider<'l>
  where
    Self: 'l;

  async fn mutate<const SIZE: usize, V>(
    &mut self,
    offset: u64,
    writer: impl AsyncFnOnce(&mut [u8; SIZE]) -> V,
  ) -> Result<V, crate::provider::error::provider_mutate::ProviderMutateError<Self::MutateError>> {
    (**self).mutate(offset, writer).await
  }

  fn mut_slice<'l, const SIZE: usize>(&'l mut self, start: u64) -> Result<Self::StaticMutSliceProvider<'l, SIZE>, ProviderSliceError<Self::SliceError>> {
    (**self).mut_slice::<SIZE>(start)
  }

  fn mut_slice_dynamic<'l>(&'l mut self, start: u64, size: Option<u64>) -> Result<Self::DynamicMutSliceProvider<'l>, ProviderSliceError<Self::SliceError>> {
    (**self).mut_slice_dynamic(start, size)
  }
}

impl<P: ResizableProvider> ResizableProvider for &mut P {
  type ResizeError = P::ResizeError;

  async fn resize_at(&mut self, offset: u64, old_len: u64, new_len: u64) -> Result<(), super::error::provider_resize::ProviderResizeError<Self::ResizeError>> {
    (**self).resize_at(offset, old_len, new_len).await
  }
}
