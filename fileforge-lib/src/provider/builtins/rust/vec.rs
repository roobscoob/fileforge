use crate::provider::{
  builtins::slice::{dynamic::DynamicSliceProvider, fixed::FixedSliceProvider}, error::{out_of_bounds::OutOfBoundsError, provider_read::ProviderReadError, provider_slice::ProviderSliceError}, hint::ReadHint, MutProvider, Provider, ResizableProvider
};

impl Provider for alloc::vec::Vec<u8> {
  type ReadError = core::convert::Infallible;
  type SliceError = core::convert::Infallible;

  type StaticSliceProvider<'l, const SIZE: usize>
    = FixedSliceProvider<SIZE, &'l alloc::vec::Vec<u8>>
  where
    Self: 'l;

  type DynamicSliceProvider<'l>
    = DynamicSliceProvider<&'l alloc::vec::Vec<u8>>
  where
    Self: 'l;

  fn len(&self) -> u64 { self.len() as u64 }

  async fn read<const SIZE: usize, V>(
    &self,
    offset: u64,
    _hint: ReadHint,
    reader: impl AsyncFnOnce(&[u8; SIZE]) -> V,
  ) -> Result<V, ProviderReadError<Self::ReadError>> {
    OutOfBoundsError::assert(self.len() as u64, offset, Some(SIZE as u64))?;

    Ok(reader(&self[offset as usize..(offset + SIZE as u64) as usize].try_into().unwrap()).await)
  }

  fn slice<'l, const SIZE: usize>(&'l self, start: u64) -> Result<Self::StaticSliceProvider<'l, SIZE>, ProviderSliceError<Self::SliceError>> {
    Ok(FixedSliceProvider::new(start, self)?)
  }

  fn slice_dynamic<'l>(
    &'l self,
    start: u64,
    size: Option<u64>,
  ) -> Result<Self::DynamicSliceProvider<'l>, ProviderSliceError<Self::SliceError>> {
    Ok(DynamicSliceProvider::new(start, size, self)?)
  }
}

impl MutProvider for alloc::vec::Vec<u8> {
  type MutateError = core::convert::Infallible;

  type DynamicMutSliceProvider<'l> = DynamicSliceProvider<&'l mut alloc::vec::Vec<u8>>
    where Self: 'l;

  type StaticMutSliceProvider<'l, const SIZE: usize> = FixedSliceProvider<SIZE, &'l mut alloc::vec::Vec<u8>>
    where Self: 'l;

  async fn mutate<const SIZE: usize, V>(
    &mut self,
    offset: u64,
    writer: impl AsyncFnOnce(&mut [u8; SIZE]) -> V,
  ) -> Result<V, crate::provider::error::provider_mutate::ProviderMutateError<Self::MutateError>> {
    OutOfBoundsError::assert(<&mut alloc::vec::Vec<u8> as Provider>::len(&self) as u64, offset, Some(SIZE as u64))?;

    Ok(writer(&mut self[offset as usize..(offset + SIZE as u64) as usize].try_into().unwrap()).await)
  }

  fn mut_slice<'l, const SIZE: usize>(
    &'l mut self,
    start: u64,
  ) -> Result<Self::StaticMutSliceProvider<'l, SIZE>, ProviderSliceError<Self::SliceError>> {
    Ok(FixedSliceProvider::new(start, self)?)
  }

  fn mut_slice_dynamic<'l>(
    &'l mut self,
    start: u64,
    size: Option<u64>,
  ) -> Result<Self::DynamicMutSliceProvider<'l>, ProviderSliceError< Self::SliceError>> {
    Ok(DynamicSliceProvider::new(start, size, self)?)
  }
}

impl ResizableProvider for alloc::vec::Vec<u8> {
  type ResizeError = core::convert::Infallible;

  async fn resize_at(&mut self, offset: u64, old_len: u64, new_len: u64) -> Result<(), crate::provider::error::provider_resize::ProviderResizeError<Self::ResizeError>> {
    OutOfBoundsError::assert(<&mut alloc::vec::Vec<u8> as Provider>::len(&self) as u64, offset, Some(old_len))?;

    if new_len == old_len {
      return Ok(());
    }

    // step 1: determine if we're growing or shrinking
    if new_len > old_len {
      // step 2: determine how much we need to grow
      let grow_by = new_len - old_len;

      // step 3: grow the vector
      self.resize((<&mut alloc::vec::Vec<u8> as Provider>::len(&self) + grow_by) as usize, 0);

      // step 4: move the data to make room for the new data
      let start = (offset + old_len) as usize;
      let end = <&mut alloc::vec::Vec<u8> as Provider>::len(&self) as usize;

      self.copy_within(start..end, start + grow_by as usize);
    } else {
      // step 2: determine how much we need to shrink
      let shrink_by = old_len - new_len;

      // step 3: move the data to fill the gap
      let start = (offset + new_len) as usize;
      let end = <&mut alloc::vec::Vec<u8> as Provider>::len(&self) as usize;

      self.copy_within(start..end, start - shrink_by as usize);

      // step 4: shrink the vector
      self.resize(new_len as usize, 0);
    }

    Ok(())
  }
}
