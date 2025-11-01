use crate::provider::{
  Provider,
  error::{out_of_bounds::OutOfBoundsError, provider_read::ProviderReadError, provider_slice::ProviderSliceError},
};

use super::{fixed_slice::RustFixedSliceProvider, slice::RustSliceProvider};

pub struct RustMutSliceProvider<'l, T> {
  data: &'l mut [T],
}

impl<'l, T> From<&'l mut [T]> for RustMutSliceProvider<'l, T> {
  fn from(data: &'l mut [T]) -> Self {
    Self { data }
  }
}

impl<'l, T> Provider for RustMutSliceProvider<'l, T> {
  type Type = T;

  type ReadError = core::convert::Infallible;
  type SliceError = core::convert::Infallible;

  type StaticSliceProvider<'l2, const SIZE: usize>
    = RustFixedSliceProvider<'l2, T, SIZE>
  where
    Self: 'l2;

  type DynamicSliceProvider<'l2>
    = RustSliceProvider<'l2, T>
  where
    Self: 'l2;

  fn len(&self) -> u64 {
    self.data.len() as u64
  }

  async fn read<const SIZE: usize, V>(&self, offset: u64, _hint: crate::provider::hint::ReadHint, reader: impl AsyncFnOnce(&[Self::Type; SIZE]) -> V) -> Result<V, ProviderReadError<Self::ReadError>> {
    OutOfBoundsError::assert(self.data.len() as u64, offset, Some(SIZE as u64))?;

    Ok(reader(&self.data[offset as usize..(offset + SIZE as u64) as usize].split_first_chunk::<SIZE>().unwrap().0).await)
  }

  fn slice<'l2, const SIZE: usize>(&'l2 self, start: u64) -> Result<Self::StaticSliceProvider<'l2, SIZE>, ProviderSliceError<Self::SliceError>> {
    OutOfBoundsError::assert(self.data.len() as u64, start, Some(SIZE as u64))?;

    let slice = &self.data[start as usize..(start + SIZE as u64) as usize];
    let slice: &[T; SIZE] = slice.try_into().unwrap();

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
