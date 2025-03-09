use core::future::ready;

use crate::{
  provider::{error::{out_of_bounds::OutOfBoundsError, provider_read::ProviderReadError, provider_slice::ProviderSliceError}, Provider},
  stream::{collectable::Collectable, error::stream_read::StreamReadError, ReadableStream},
};

use super::{fixed_slice::RustFixedSliceProvider, slice::RustSliceProvider};

pub struct RustMutSliceProvider<'l> {
  data: &'l mut [u8],
}

impl<'l> From<&'l mut [u8]> for RustMutSliceProvider<'l> {
  fn from(data: &'l mut [u8]) -> Self { Self { data } }
}

impl<'l> Provider for RustMutSliceProvider<'l> {
  type ReadError = core::convert::Infallible;
  type SliceError = core::convert::Infallible;

  type StaticSliceProvider<'l2, const SIZE: usize>
    = RustFixedSliceProvider<'l2, SIZE>
  where
    Self: 'l2;

  type DynamicSliceProvider<'l2>
    = RustSliceProvider<'l2>
  where
    Self: 'l2;

  fn len(&self) -> u64 { self.data.len() as u64 }

  async fn read<const SIZE: usize, V>(
    &self,
    offset: u64,
    _hint: crate::provider::hint::ReadHint,
    reader: impl AsyncFnOnce(&[u8; SIZE]) -> V,
  ) -> Result<V, ProviderReadError<Self::ReadError>> {
    OutOfBoundsError::assert(self.data.len() as u64, offset, Some(SIZE as u64))?;

    Ok(reader(&self.data[offset as usize..(offset + SIZE as u64) as usize].try_into().unwrap()).await)
  }

  fn slice<'l2, const SIZE: usize>(
    &'l2 self,
    start: u64,
  ) -> Result<Self::StaticSliceProvider<'l2, SIZE>, ProviderSliceError<Self::SliceError>> {
    OutOfBoundsError::assert(self.data.len() as u64, start, Some(SIZE as u64))?;

    let slice = &self.data[start as usize..(start + SIZE as u64) as usize];
    let slice: &[u8; SIZE] = slice.try_into().unwrap();

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

impl<'l, S: ReadableStream> Collectable<S> for RustMutSliceProvider<'l> {
  type Error = StreamReadError<S::ReadError>;

  async fn collect(&mut self, stream: &mut S) -> Result<(), Self::Error> {
    for i in 0..self.data.len() {
      self.data[i] = stream.read(|v: &[u8; 1]| ready(v[0])).await?;
    }

    Ok(())
  }
}
