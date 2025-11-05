use crate::provider::{
  error::{out_of_bounds::OutOfBoundsError, provider_read::ProviderReadError, provider_slice::ProviderSliceError},
  Provider,
};

impl<T> Provider for [T]
where
  T: Copy,
{
  type Type = T;

  type ReadError = core::convert::Infallible;
  type SliceError = core::convert::Infallible;

  type StaticSliceProvider<'a, const SIZE: usize>
    = &'a [T; SIZE]
  where
    T: 'a;

  type DynamicSliceProvider<'l2>
    = &'l2 [T]
  where
    Self: 'l2;

  fn len(&self) -> u64 {
    self.len() as u64
  }

  async fn read<const SIZE: usize, V>(&self, offset: u64, _hint: crate::provider::hint::ReadHint, reader: impl AsyncFnOnce(&[Self::Type; SIZE]) -> V) -> Result<V, ProviderReadError<Self::ReadError>> {
    OutOfBoundsError::assert(self.len() as u64, offset, Some(SIZE as u64))?;

    Ok(reader(&self[offset as usize..(offset + SIZE as u64) as usize].split_first_chunk::<SIZE>().unwrap().0).await)
  }

  fn slice<'a, const SIZE: usize>(&'a self, start: u64) -> Result<Self::StaticSliceProvider<'a, SIZE>, ProviderSliceError<Self::SliceError>> {
    OutOfBoundsError::assert(self.len() as u64, start, Some(SIZE as u64))?;

    let slice = &self[start as usize..(start + SIZE as u64) as usize];
    let slice: &[T; SIZE] = slice.try_into().unwrap();

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
