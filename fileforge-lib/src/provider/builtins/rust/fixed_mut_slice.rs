use core::future::ready;

use crate::{
  provider::{error::{out_of_bounds::OutOfBoundsError, provider_read::ProviderReadError, provider_slice::ProviderSliceError}, Provider},
  stream::{collectable::Collectable, error::stream_read::StreamReadError},
};

use super::{fixed_slice::RustFixedSliceProvider, slice::RustSliceProvider};

pub struct RustFixedMutSliceProvider<'l, const SIZE: usize> {
  data: &'l mut [u8; SIZE],
}

impl<'l, const SIZE: usize> From<&'l mut [u8; SIZE]> for RustFixedMutSliceProvider<'l, SIZE> {
  fn from(data: &'l mut [u8; SIZE]) -> Self { Self { data } }
}

impl<'l, const SIZE: usize> Provider for RustFixedMutSliceProvider<'l, SIZE> {
  type ReadError = core::convert::Infallible;
  type SliceError = core::convert::Infallible;

  type StaticSliceProvider<'l2, const SLICE_SIZE: usize>
    = RustFixedSliceProvider<'l2, SLICE_SIZE>
  where
    Self: 'l2;

  type DynamicSliceProvider<'l2>
    = RustSliceProvider<'l2>
  where
    Self: 'l2;

  fn len(&self) -> u64 { SIZE as u64 }

  async fn read<const READ_SIZE: usize, V>(
    &self,
    offset: u64,
    _hint: crate::provider::hint::ReadHint,
    reader: impl AsyncFnOnce(&[u8; READ_SIZE]) -> V,
  ) -> Result<V, ProviderReadError<Self::ReadError>> {
    OutOfBoundsError::assert(SIZE as u64, offset, Some(READ_SIZE as u64))?;

    Ok(reader(&self.data[offset as usize..(offset + READ_SIZE as u64) as usize].try_into().unwrap()).await)
  }

  fn slice<'l2, const SLICE_SIZE: usize>(
    &'l2 self,
    start: u64,
  ) -> Result<Self::StaticSliceProvider<'l2, SLICE_SIZE>, ProviderSliceError<Self::SliceError>> {
    OutOfBoundsError::assert(SIZE as u64, start, Some(SLICE_SIZE as u64))?;

    let slice = &self.data[start as usize..(start + SLICE_SIZE as u64) as usize];
    let slice: &[u8; SLICE_SIZE] = slice.try_into().unwrap();

    Ok(slice.into())
  }

  fn slice_dynamic<'l2>(
    &'l2 self,
    start: u64,
    size: Option<u64>,
  ) -> Result<Self::DynamicSliceProvider<'l2>, ProviderSliceError<Self::SliceError>> {
    OutOfBoundsError::assert(self.data.len() as u64, start, size)?;

    let slice = if let Some(size) = size {
      &self.data[start as usize..(start + size) as usize]
    } else {
      &self.data[start as usize..]
    };

    Ok(slice.into())
  }
}

impl<'l, const SIZE: usize, S: crate::stream::ReadableStream> Collectable<S> for RustFixedMutSliceProvider<'l, SIZE> {
  type Error = StreamReadError<S::ReadError>;

  async fn collect(&mut self, stream: &mut S) -> Result<(), Self::Error> { stream.read(|data: &[u8; SIZE]| ready(self.data.copy_from_slice(data))).await }
}
