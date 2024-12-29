use crate::provider::{error::out_of_bounds::OutOfBoundsError, MutProvider, Provider};

pub struct FixedSliceProvider<const NODE_NAME_SIZE: usize, const SIZE: usize, UnderlyingProvider: Provider<NODE_NAME_SIZE>> {
  offset: u64,
  provider: UnderlyingProvider,
}

impl<const NODE_NAME_SIZE: usize, const SIZE: usize, UnderlyingProvider: Provider<NODE_NAME_SIZE>> FixedSliceProvider<NODE_NAME_SIZE, SIZE, UnderlyingProvider> {
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

impl<const NODE_NAME_SIZE: usize, const SIZE: usize, UnderlyingProvider: Provider<NODE_NAME_SIZE>> Provider<NODE_NAME_SIZE> for FixedSliceProvider<NODE_NAME_SIZE, SIZE, UnderlyingProvider> {
  fn len(&self) -> u64 { SIZE as u64 }

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

  async fn read<const READ_SIZE: usize, V, R: core::future::Future<Output = V>>(
    &self,
    offset: u64,
    hint: crate::provider::hint::ReadHint,
    reader: impl FnOnce(&[u8; READ_SIZE]) -> R,
  ) -> Result<V, crate::provider::error::provider_read::ProviderReadError<NODE_NAME_SIZE, Self::ReadError>> {
    OutOfBoundsError::assert(SIZE as u64, offset, Some(READ_SIZE as u64))?;

    self.provider.read(offset + self.offset, hint, reader).await
  }

  fn slice<'l, const SLICE_SIZE: usize>(
    &'l self,
    start: u64,
  ) -> Result<Self::StaticSliceProvider<'l, SLICE_SIZE>, crate::provider::error::provider_slice::ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>> {
    OutOfBoundsError::assert(SIZE as u64, start, Some(SLICE_SIZE as u64))?;

    self.provider.slice(start + self.offset)
  }

  fn slice_dynamic<'l>(
    &'l self,
    start: u64,
    size: Option<u64>,
  ) -> Result<Self::DynamicSliceProvider<'l>, crate::provider::error::provider_slice::ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>> {
    OutOfBoundsError::assert(SIZE as u64, start, size)?;

    self.provider.slice_dynamic(start + self.offset, size)
  }
}

impl<const NODE_NAME_SIZE: usize, const SIZE: usize, UnderlyingProvider: MutProvider<NODE_NAME_SIZE>> MutProvider<NODE_NAME_SIZE> for FixedSliceProvider<NODE_NAME_SIZE, SIZE, UnderlyingProvider> {
  type MutateError = UnderlyingProvider::MutateError;

  type DynamicMutSliceProvider<'l>
    = UnderlyingProvider::DynamicMutSliceProvider<'l>
  where
    Self: 'l;

  type StaticMutSliceProvider<'l, const SLICE_SIZE: usize>
    = UnderlyingProvider::StaticMutSliceProvider<'l, SLICE_SIZE>
  where
    Self: 'l;

  async fn mutate<const MUTATE_SIZE: usize, V, R: core::future::Future<Output = V>>(
    &mut self,
    offset: u64,
    writer: impl FnOnce(&mut [u8; MUTATE_SIZE]) -> R,
  ) -> Result<V, crate::provider::error::provider_mutate::ProviderMutateError<NODE_NAME_SIZE, Self::MutateError>> {
    OutOfBoundsError::assert(SIZE as u64, offset, Some(MUTATE_SIZE as u64))?;

    self.provider.mutate(offset + self.offset, writer).await
  }

  fn mut_slice<'l, const SLICE_SIZE: usize>(
    &'l mut self,
    start: u64,
  ) -> Result<Self::StaticMutSliceProvider<'l, SLICE_SIZE>, crate::provider::error::provider_slice::ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>> {
    OutOfBoundsError::assert(SIZE as u64, start, Some(SIZE as u64))?;

    self.provider.mut_slice(start + self.offset)
  }

  fn mut_slice_dynamic<'l>(
    &'l mut self,
    start: u64,
    size: Option<u64>,
  ) -> Result<Self::DynamicMutSliceProvider<'l>, crate::provider::error::provider_slice::ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>> {
    OutOfBoundsError::assert(SIZE as u64, start, size)?;

    self.provider.mut_slice_dynamic(start + self.offset, size)
  }
}
