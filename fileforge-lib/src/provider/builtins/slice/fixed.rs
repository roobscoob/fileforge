use crate::provider::{
  MutProvider, Provider,
  error::{out_of_bounds::OutOfBoundsError, provider_mutate::ProviderMutateError, provider_read::ProviderReadError, provider_slice::ProviderSliceError},
};

pub struct FixedSliceProvider<const SIZE: usize, UnderlyingProvider: Provider> {
  offset: u64,
  provider: UnderlyingProvider,
}

impl<const SIZE: usize, UnderlyingProvider: Provider> FixedSliceProvider<SIZE, UnderlyingProvider> {
  pub fn new(offset: u64, provider: UnderlyingProvider) -> Result<Self, OutOfBoundsError> {
    let slice_end = offset.checked_add(SIZE as u64).ok_or(OutOfBoundsError {
      read_offset: offset,
      read_length: Some(SIZE as u64),
      provider_size: provider.len(),
    })?;

    if slice_end > provider.len() {
      return Err(OutOfBoundsError {
        read_offset: offset,
        read_length: Some(SIZE as u64),
        provider_size: provider.len(),
      });
    }

    Ok(Self { offset, provider })
  }
}

impl<const SIZE: usize, UnderlyingProvider: Provider> Provider for FixedSliceProvider<SIZE, UnderlyingProvider> {
  type Type = UnderlyingProvider::Type;

  fn len(&self) -> u64 {
    SIZE as u64
  }

  type ReadError = UnderlyingProvider::ReadError;
  type SliceError = UnderlyingProvider::SliceError;

  type StaticSliceProvider<'l, const SLICE_SIZE: usize>
    = UnderlyingProvider::StaticSliceProvider<'l, SLICE_SIZE>
  where
    Self: 'l;

  type DynamicSliceProvider<'l>
    = UnderlyingProvider::DynamicSliceProvider<'l>
  where
    Self: 'l;

  async fn read<const READ_SIZE: usize, V>(
    &self,
    offset: u64,
    hint: crate::provider::hint::ReadHint,
    reader: impl AsyncFnOnce(&[Self::Type; READ_SIZE]) -> V,
  ) -> Result<V, ProviderReadError<Self::ReadError>> {
    OutOfBoundsError::assert(SIZE as u64, offset, Some(READ_SIZE as u64))?;

    self.provider.read(offset + self.offset, hint, reader).await
  }

  fn slice<'l, const SLICE_SIZE: usize>(&'l self, start: u64) -> Result<Self::StaticSliceProvider<'l, SLICE_SIZE>, ProviderSliceError<Self::SliceError>> {
    OutOfBoundsError::assert(SIZE as u64, start, Some(SLICE_SIZE as u64))?;

    self.provider.slice(start + self.offset)
  }

  fn slice_dynamic<'l>(&'l self, start: u64, size: Option<u64>) -> Result<Self::DynamicSliceProvider<'l>, ProviderSliceError<Self::SliceError>> {
    OutOfBoundsError::assert(SIZE as u64, start, size)?;

    self.provider.slice_dynamic(start + self.offset, size)
  }
}

impl<const SIZE: usize, UnderlyingProvider: MutProvider> MutProvider for FixedSliceProvider<SIZE, UnderlyingProvider> {
  type MutateError = UnderlyingProvider::MutateError;

  type DynamicMutSliceProvider<'l>
    = UnderlyingProvider::DynamicMutSliceProvider<'l>
  where
    Self: 'l;

  type StaticMutSliceProvider<'l, const SLICE_SIZE: usize>
    = UnderlyingProvider::StaticMutSliceProvider<'l, SLICE_SIZE>
  where
    Self: 'l;

  async fn mutate<const MUTATE_SIZE: usize, V>(&mut self, offset: u64, writer: impl AsyncFnOnce(&mut [Self::Type; MUTATE_SIZE]) -> V) -> Result<V, ProviderMutateError<Self::MutateError>> {
    OutOfBoundsError::assert(SIZE as u64, offset, Some(MUTATE_SIZE as u64))?;

    self.provider.mutate(offset + self.offset, writer).await
  }

  fn mut_slice<'l, const SLICE_SIZE: usize>(&'l mut self, start: u64) -> Result<Self::StaticMutSliceProvider<'l, SLICE_SIZE>, ProviderSliceError<Self::SliceError>> {
    OutOfBoundsError::assert(SIZE as u64, start, Some(SIZE as u64))?;

    self.provider.mut_slice(start + self.offset)
  }

  fn mut_slice_dynamic<'l>(&'l mut self, start: u64, size: Option<u64>) -> Result<Self::DynamicMutSliceProvider<'l>, crate::provider::error::provider_slice::ProviderSliceError<Self::SliceError>> {
    OutOfBoundsError::assert(SIZE as u64, start, size)?;

    self.provider.mut_slice_dynamic(start + self.offset, size)
  }
}
