use crate::provider::{
  Provider,
  error::{out_of_bounds::OutOfBoundsError, provider_read::ProviderReadError, provider_slice::ProviderSliceError},
};

use super::{fixed_slice::RustFixedSliceProvider, slice::RustSliceProvider};

pub struct RustFixedMutSliceProvider<'l, T, const SIZE: usize> {
  data: &'l mut [T; SIZE],
}

impl<'l, T, const SIZE: usize> From<&'l mut [T; SIZE]> for RustFixedMutSliceProvider<'l, T, SIZE> {
  fn from(data: &'l mut [T; SIZE]) -> Self {
    Self { data }
  }
}

impl<'l, T, const SIZE: usize> Provider for RustFixedMutSliceProvider<'l, T, SIZE> {
  type Type = T;

  type ReadError = core::convert::Infallible;
  type SliceError = core::convert::Infallible;

  type StaticSliceProvider<'l2, const SLICE_SIZE: usize>
    = RustFixedSliceProvider<'l2, T, SLICE_SIZE>
  where
    Self: 'l2;

  type DynamicSliceProvider<'l2>
    = RustSliceProvider<'l2, T>
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

    Ok(reader(&self.data[offset as usize..(offset + READ_SIZE as u64) as usize].split_first_chunk::<READ_SIZE>().unwrap().0).await)
  }

  fn slice<'l2, const SLICE_SIZE: usize>(&'l2 self, start: u64) -> Result<Self::StaticSliceProvider<'l2, SLICE_SIZE>, ProviderSliceError<Self::SliceError>> {
    OutOfBoundsError::assert(SIZE as u64, start, Some(SLICE_SIZE as u64))?;

    let slice = &self.data[start as usize..(start + SLICE_SIZE as u64) as usize];
    let slice: &[T; SLICE_SIZE] = slice.try_into().unwrap();

    Ok(slice.into())
  }

  fn slice_dynamic<'l2>(&'l2 self, start: u64, size: Option<u64>) -> Result<Self::DynamicSliceProvider<'l2>, ProviderSliceError<Self::SliceError>> {
    OutOfBoundsError::assert(self.data.len() as u64, start, size)?;

    let slice = if let Some(size) = size {
      &self.data[start as usize..(start + size) as usize]
    } else {
      &self.data[start as usize..]
    };

    Ok(slice.into())
  }
}
