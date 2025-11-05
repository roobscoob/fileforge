use crate::provider::{
  error::{out_of_bounds::OutOfBoundsError, provider_read::ProviderReadError, provider_slice::ProviderSliceError},
  Provider,
};

impl<T, const SIZE: usize> Provider for [T; SIZE]
where
  T: Copy,
{
  type Type = T;

  type ReadError = core::convert::Infallible;
  type SliceError = core::convert::Infallible;

  type StaticSliceProvider<'a, const SLICE_SIZE: usize>
    = &'a [T; SLICE_SIZE]
  where
    T: 'a;

  type DynamicSliceProvider<'l2>
    = &'l2 [T]
  where
    Self: 'l2;

  fn len(&self) -> u64 {
    SIZE as u64
  }

  async fn read<const READ_SIZE: usize, V>(
    &self,
    offset: u64,
    _hint: crate::provider::hint::ReadHint,
    reader: impl AsyncFnOnce(&[T; READ_SIZE]) -> V,
  ) -> Result<V, ProviderReadError<Self::ReadError>> {
    OutOfBoundsError::assert(SIZE as u64, offset, Some(READ_SIZE as u64))?;

    Ok(reader(&self[offset as usize..(offset + READ_SIZE as u64) as usize].split_first_chunk::<READ_SIZE>().unwrap().0).await)
  }

  fn slice<'a, const SLICE_SIZE: usize>(&'a self, start: u64) -> Result<Self::StaticSliceProvider<'a, SLICE_SIZE>, ProviderSliceError<Self::SliceError>> {
    OutOfBoundsError::assert(SIZE as u64, start, Some(SLICE_SIZE as u64))?;

    let slice = &self[start as usize..(start + SLICE_SIZE as u64) as usize];
    let slice: &[T; SLICE_SIZE] = slice.try_into().unwrap();

    Ok(slice)
  }

  fn slice_dynamic<'l2>(&'l2 self, start: u64, size: Option<u64>) -> Result<Self::DynamicSliceProvider<'l2>, ProviderSliceError<Self::SliceError>> {
    OutOfBoundsError::assert(self.len() as u64, start, size)?;

    let slice = if let Some(size) = size {
      &self[start as usize..(start + size) as usize]
    } else {
      &self[start as usize..]
    };

    Ok(slice.into())
  }
}
