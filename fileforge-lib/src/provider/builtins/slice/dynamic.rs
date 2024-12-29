use crate::provider::{error::out_of_bounds::OutOfBoundsError, MutProvider, Provider, ResizableProvider};

pub struct DynamicSliceProvider<const NODE_NAME_SIZE: usize, UnderlyingProvider: Provider<NODE_NAME_SIZE>> {
  offset: u64,
  size: Option<u64>,
  provider: UnderlyingProvider,
}

impl<const NODE_NAME_SIZE: usize, UnderlyingProvider: Provider<NODE_NAME_SIZE>> DynamicSliceProvider<NODE_NAME_SIZE, UnderlyingProvider> {
  pub fn new(offset: u64, size: Option<u64>, provider: UnderlyingProvider) -> Result<Self, OutOfBoundsError> {
    if let Some(size) = size {
      let slice_end = offset.checked_add(size).ok_or(OutOfBoundsError {
        read_offset: offset,
        read_length: Some(size),
        provider_size: provider.len(),
      })?;

      if slice_end > provider.len() {
        return Err(OutOfBoundsError {
          read_offset: offset,
          read_length: Some(size),
          provider_size: provider.len(),
        });
      }
    } else {
      if offset > provider.len() {
        return Err(OutOfBoundsError {
          read_offset: offset,
          read_length: None,
          provider_size: provider.len(),
        });
      }
    }

    Ok(Self { offset, size, provider })
  }
}

impl<const NODE_NAME_SIZE: usize, UnderlyingProvider: Provider<NODE_NAME_SIZE>> Provider<NODE_NAME_SIZE> for DynamicSliceProvider<NODE_NAME_SIZE, UnderlyingProvider> {
  fn len(&self) -> u64 { self.size.unwrap_or_else(|| self.provider.len() - self.offset) }

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
    OutOfBoundsError::assert(self.len(), offset, Some(READ_SIZE as u64))?;

    self.provider.read(offset + self.offset, hint, reader).await
  }

  fn slice<'l, const SLICE_SIZE: usize>(
    &'l self,
    start: u64,
  ) -> Result<Self::StaticSliceProvider<'l, SLICE_SIZE>, crate::provider::error::provider_slice::ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>> {
    OutOfBoundsError::assert(self.len(), start, Some(SLICE_SIZE as u64))?;

    self.provider.slice(start + self.offset)
  }

  fn slice_dynamic<'l>(
    &'l self,
    start: u64,
    size: Option<u64>,
  ) -> Result<Self::DynamicSliceProvider<'l>, crate::provider::error::provider_slice::ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>> {
    OutOfBoundsError::assert(self.len(), start, size)?;

    self.provider.slice_dynamic(start + self.offset, size)
  }
}

impl<const NODE_NAME_SIZE: usize, UnderlyingProvider: MutProvider<NODE_NAME_SIZE>> MutProvider<NODE_NAME_SIZE> for DynamicSliceProvider<NODE_NAME_SIZE, UnderlyingProvider> {
  type MutateError = UnderlyingProvider::MutateError;

  type StaticMutSliceProvider<'l, const SIZE: usize>
    = UnderlyingProvider::StaticMutSliceProvider<'l, SIZE>
  where
    Self: 'l;

  type DynamicMutSliceProvider<'l>
    = UnderlyingProvider::DynamicMutSliceProvider<'l>
  where
    Self: 'l;

  async fn mutate<const SIZE: usize, V, R: core::future::Future<Output = V>>(
    &mut self,
    offset: u64,
    writer: impl FnOnce(&mut [u8; SIZE]) -> R,
  ) -> Result<V, crate::provider::error::provider_mutate::ProviderMutateError<NODE_NAME_SIZE, Self::MutateError>> {
    OutOfBoundsError::assert(self.len(), offset, Some(SIZE as u64))?;

    self.provider.mutate(offset + self.offset, writer).await
  }

  fn mut_slice<'l, const SIZE: usize>(
    &'l mut self,
    start: u64,
  ) -> Result<Self::StaticMutSliceProvider<'l, SIZE>, crate::provider::error::provider_slice::ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>> {
    OutOfBoundsError::assert(self.len(), start, Some(SIZE as u64))?;

    self.provider.mut_slice(start + self.offset)
  }

  fn mut_slice_dynamic<'l>(
    &'l mut self,
    start: u64,
    size: Option<u64>,
  ) -> Result<Self::DynamicMutSliceProvider<'l>, crate::provider::error::provider_slice::ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>> {
    OutOfBoundsError::assert(self.len(), start, size)?;

    self.provider.mut_slice_dynamic(start + self.offset, size)
  }
}

impl<const NODE_NAME_SIZE: usize, UnderlyingProvider: ResizableProvider<NODE_NAME_SIZE>> ResizableProvider<NODE_NAME_SIZE> for DynamicSliceProvider<NODE_NAME_SIZE, UnderlyingProvider> {
  type ResizeError = UnderlyingProvider::ResizeError;

  async fn resize_at(&mut self, offset: u64, old_len: u64, new_len: u64) -> Result<(), crate::provider::error::provider_resize::ProviderResizeError<NODE_NAME_SIZE, Self::ResizeError>> {
    OutOfBoundsError::assert(self.len(), offset, Some(old_len))?;

    self.provider.resize_at(offset + self.offset, old_len, new_len).await?;

    if let Some(ref mut size) = self.size {
      *size += new_len - old_len;
    }

    Ok(())
  }
}
