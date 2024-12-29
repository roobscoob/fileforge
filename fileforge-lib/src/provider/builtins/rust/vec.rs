use crate::provider::{
  builtins::slice::{dynamic::DynamicSliceProvider, fixed::FixedSliceProvider},
  error::out_of_bounds::OutOfBoundsError,
  MutProvider, Provider, ResizableProvider,
};

impl<const NODE_NAME_SIZE: usize> Provider<NODE_NAME_SIZE> for alloc::vec::Vec<u8> {
  type ReadError = core::convert::Infallible;
  type SliceError = core::convert::Infallible;

  type StaticSliceProvider<'l, const SIZE: usize>
    = FixedSliceProvider<NODE_NAME_SIZE, SIZE, &'l alloc::vec::Vec<u8>>
  where
    Self: 'l;

  type DynamicSliceProvider<'l>
    = DynamicSliceProvider<NODE_NAME_SIZE, &'l alloc::vec::Vec<u8>>
  where
    Self: 'l;

  fn len(&self) -> u64 { self.len() as u64 }

  async fn read<const SIZE: usize, V, R: core::future::Future<Output = V>>(
    &self,
    offset: u64,
    _hint: crate::provider::hint::ReadHint,
    reader: impl FnOnce(&[u8; SIZE]) -> R,
  ) -> Result<V, crate::provider::error::provider_read::ProviderReadError<NODE_NAME_SIZE, Self::ReadError>> {
    OutOfBoundsError::assert(self.len() as u64, offset, Some(SIZE as u64))?;

    Ok(reader(&self[offset as usize..(offset + SIZE as u64) as usize].try_into().unwrap()).await)
  }

  fn slice<'l, const SIZE: usize>(&'l self, start: u64) -> Result<Self::StaticSliceProvider<'l, SIZE>, crate::provider::error::provider_slice::ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>> {
    Ok(FixedSliceProvider::new(start, self)?)
  }

  fn slice_dynamic<'l>(
    &'l self,
    start: u64,
    size: Option<u64>,
  ) -> Result<Self::DynamicSliceProvider<'l>, crate::provider::error::provider_slice::ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>> {
    Ok(DynamicSliceProvider::new(start, size, self)?)
  }
}

impl<const NODE_NAME_SIZE: usize> MutProvider<NODE_NAME_SIZE> for alloc::vec::Vec<u8> {
  type MutateError = core::convert::Infallible;

  type DynamicMutSliceProvider<'l>
    = DynamicSliceProvider<NODE_NAME_SIZE, &'l mut alloc::vec::Vec<u8>>
  where
    Self: 'l;

  type StaticMutSliceProvider<'l, const SIZE: usize>
    = FixedSliceProvider<NODE_NAME_SIZE, SIZE, &'l mut alloc::vec::Vec<u8>>
  where
    Self: 'l;

  async fn mutate<const SIZE: usize, V, R: core::future::Future<Output = V>>(
    &mut self,
    offset: u64,
    writer: impl FnOnce(&mut [u8; SIZE]) -> R,
  ) -> Result<V, crate::provider::error::provider_mutate::ProviderMutateError<NODE_NAME_SIZE, Self::MutateError>> {
    OutOfBoundsError::assert(<&mut alloc::vec::Vec<u8> as Provider<NODE_NAME_SIZE>>::len(&self) as u64, offset, Some(SIZE as u64))?;

    Ok(writer(&mut self[offset as usize..(offset + SIZE as u64) as usize].try_into().unwrap()).await)
  }

  fn mut_slice<'l, const SIZE: usize>(
    &'l mut self,
    start: u64,
  ) -> Result<Self::StaticMutSliceProvider<'l, SIZE>, crate::provider::error::provider_slice::ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>> {
    Ok(FixedSliceProvider::new(start, self)?)
  }

  fn mut_slice_dynamic<'l>(
    &'l mut self,
    start: u64,
    size: Option<u64>,
  ) -> Result<Self::DynamicMutSliceProvider<'l>, crate::provider::error::provider_slice::ProviderSliceError<NODE_NAME_SIZE, Self::SliceError>> {
    Ok(DynamicSliceProvider::new(start, size, self)?)
  }
}

impl<const NODE_NAME_SIZE: usize> ResizableProvider<NODE_NAME_SIZE> for alloc::vec::Vec<u8> {
  type ResizeError = core::convert::Infallible;

  async fn resize_at(&mut self, offset: u64, old_len: u64, new_len: u64) -> Result<(), crate::provider::error::provider_resize::ProviderResizeError<NODE_NAME_SIZE, Self::ResizeError>> {
    OutOfBoundsError::assert(<&mut alloc::vec::Vec<u8> as Provider<NODE_NAME_SIZE>>::len(&self) as u64, offset, Some(old_len))?;

    if new_len == old_len {
      return Ok(());
    }

    // step 1: determine if we're growing or shrinking
    if new_len > old_len {
      // step 2: determine how much we need to grow
      let grow_by = new_len - old_len;

      // step 3: grow the vector
      self.resize((<&mut alloc::vec::Vec<u8> as Provider<NODE_NAME_SIZE>>::len(&self) + grow_by) as usize, 0);

      // step 4: move the data to make room for the new data
      let start = (offset + old_len) as usize;
      let end = <&mut alloc::vec::Vec<u8> as Provider<NODE_NAME_SIZE>>::len(&self) as usize;

      self.copy_within(start..end, start + grow_by as usize);
    } else {
      // step 2: determine how much we need to shrink
      let shrink_by = old_len - new_len;

      // step 3: move the data to fill the gap
      let start = (offset + new_len) as usize;
      let end = <&mut alloc::vec::Vec<u8> as Provider<NODE_NAME_SIZE>>::len(&self) as usize;

      self.copy_within(start..end, start - shrink_by as usize);

      // step 4: shrink the vector
      self.resize(new_len as usize, 0);
    }

    Ok(())
  }
}
