use core::cmp::min;

use crate::{
  provider::{
    error::{
      never::Never, read_error::ReadError, slice_error::SliceError, write_error::WriteError,
    },
    out_of_bounds::SliceOutOfBoundsError,
    r#trait::{MutProvider, Provider},
  },
  reader::error::underlying_provider_stat::UnderlyingProviderStatError,
};

use super::dynamic::{DynamicMutSliceProvider, DynamicSliceProvider};

pub struct FixedMutSliceProvider<'underlying, const SIZE: usize, T: Provider> {
  pub(crate) underlying_provider: &'underlying mut T,
  pub(crate) offset: u64,
}

#[derive(Clone)]
pub struct FixedSliceProvider<'underlying, const SIZE: usize, T: Provider> {
  pub(crate) underlying_provider: &'underlying T,
  pub(crate) offset: u64,
}

impl<'underlying, const SIZE: usize, T: Provider> FixedSliceProvider<'underlying, SIZE, T> {
  pub fn over(
    underlying: &'underlying T,
    offset: u64,
  ) -> Result<FixedSliceProvider<'underlying, SIZE, T>, SliceError<T::StatError>> {
    SliceOutOfBoundsError::assert_in_bounds(
      offset,
      Some(SIZE as u64),
      underlying
        .len()
        .map_err(|e| UnderlyingProviderStatError(e))?,
    )?;

    Ok(Self {
      offset,
      underlying_provider: underlying,
    })
  }
}

impl<'underlying, const SELF_SIZE: usize, UnderlyingProvider: Provider> Provider
  for FixedSliceProvider<'underlying, SELF_SIZE, UnderlyingProvider>
{
  type StatError = UnderlyingProvider::StatError;
  type ReadError = UnderlyingProvider::ReadError;
  type ReturnedProviderType<'a, const SIZE: usize>
    = FixedSliceProvider<'a, SIZE, UnderlyingProvider>
  where
    Self: 'a;
  type DynReturnedProviderType<'a>
    = DynamicSliceProvider<'a, UnderlyingProvider>
  where
    Self: 'a;

  fn slice<const SIZE: usize>(
    &self,
    offset: u64,
  ) -> Result<Self::ReturnedProviderType<'_, SIZE>, SliceError<Self::StatError>> {
    SliceOutOfBoundsError::assert_in_bounds(offset, Some(SIZE as u64), SELF_SIZE as u64)?;

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a FixedSliceProvider to exist where self.offset + SELF_SIZE would overflow
        and because offset will for sure be less than or equal to (in the case of a ZST) SELF_SIZE, this too will never overflow.
    */
    Ok(FixedSliceProvider {
      underlying_provider: self.underlying_provider,
      offset: self.offset + offset,
    })
  }

  fn slice_dyn(
    &self,
    offset: u64,
    size: Option<u64>,
  ) -> Result<Self::DynReturnedProviderType<'_>, SliceError<Self::StatError>> {
    SliceOutOfBoundsError::assert_in_bounds(offset, size, SELF_SIZE as u64)?;

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a FixedSliceProvider to exist where self.offset + SELF_SIZE would overflow
        and because offset will for sure be less than or equal to (in the case of a ZST) SELF_SIZE, this too will never overflow.
    */
    Ok(DynamicSliceProvider {
      underlying_provider: self.underlying_provider,
      offset: self.offset + offset,
      size,
    })
  }

  fn with_read<const SIZE: usize, T, CB: for<'a> FnOnce(&'a [u8; SIZE]) -> T>(
    &self,
    offset: u64,
    callback: CB,
  ) -> Result<Result<T, SliceError<Self::StatError>>, ReadError<Self::ReadError>> {
    if let Err(e) =
      SliceOutOfBoundsError::assert_in_bounds(offset, Some(SIZE as u64), SELF_SIZE as u64)
    {
      return Ok(Err(SliceError::OutOfBounds(e)));
    }

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a FixedSliceProvider to exist where self.offset + SELF_SIZE would overflow
        and because offset will for sure be less than or equal to (in the case of a ZST) SELF_SIZE, this too will never overflow.
    */
    self
      .underlying_provider
      .with_read(self.offset + offset, callback)
  }

  fn with_read_dyn<T, CB: for<'a> FnOnce(&'a [u8]) -> T>(
    &self,
    offset: u64,
    size: Option<u64>,
    callback: CB,
  ) -> Result<Result<T, SliceError<Self::StatError>>, ReadError<Self::ReadError>> {
    if let Err(e) = SliceOutOfBoundsError::assert_in_bounds(offset, size, SELF_SIZE as u64) {
      return Ok(Err(SliceError::OutOfBounds(e)));
    }

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a FixedSliceProvider to exist where self.offset + SELF_SIZE would overflow
        and because offset will for sure be less than or equal to (in the case of a ZST) SELF_SIZE, this too will never overflow.
    */
    self
      .underlying_provider
      .with_read_dyn(self.offset + offset, size, callback)
  }

  fn len(&self) -> Result<u64, Self::StatError> {
    Ok(min(
      SELF_SIZE as u64,
      self.underlying_provider.len()? - self.offset,
    ))
  }
}

impl<'underlying, const SELF_SIZE: usize, UnderlyingProvider: Provider> Provider
  for FixedMutSliceProvider<'underlying, SELF_SIZE, UnderlyingProvider>
{
  type StatError = UnderlyingProvider::StatError;
  type ReadError = UnderlyingProvider::ReadError;
  type ReturnedProviderType<'a, const SIZE: usize>
    = FixedSliceProvider<'a, SIZE, UnderlyingProvider>
  where
    Self: 'a;
  type DynReturnedProviderType<'a>
    = DynamicSliceProvider<'a, UnderlyingProvider>
  where
    Self: 'a;

  fn slice<const SIZE: usize>(
    &self,
    offset: u64,
  ) -> Result<Self::ReturnedProviderType<'_, SIZE>, SliceError<Self::StatError>> {
    SliceOutOfBoundsError::assert_in_bounds(offset, Some(SIZE as u64), SELF_SIZE as u64)?;

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a FixedSliceProvider to exist where self.offset + SELF_SIZE would overflow
        and because offset will for sure be less than or equal to (in the case of a ZST) SELF_SIZE, this too will never overflow.
    */
    Ok(FixedSliceProvider {
      underlying_provider: self.underlying_provider,
      offset: self.offset + offset,
    })
  }

  fn slice_dyn(
    &self,
    offset: u64,
    size: Option<u64>,
  ) -> Result<Self::DynReturnedProviderType<'_>, SliceError<Self::StatError>> {
    SliceOutOfBoundsError::assert_in_bounds(offset, size, SELF_SIZE as u64)?;

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a FixedSliceProvider to exist where self.offset + SELF_SIZE would overflow
        and because offset will for sure be less than or equal to (in the case of a ZST) SELF_SIZE, this too will never overflow.
    */
    Ok(DynamicSliceProvider {
      underlying_provider: self.underlying_provider,
      offset: self.offset + offset,
      size,
    })
  }

  fn with_read<const SIZE: usize, T, CB: for<'a> FnOnce(&'a [u8; SIZE]) -> T>(
    &self,
    offset: u64,
    callback: CB,
  ) -> Result<Result<T, SliceError<Self::StatError>>, ReadError<Self::ReadError>> {
    if let Err(e) =
      SliceOutOfBoundsError::assert_in_bounds(offset, Some(SIZE as u64), SELF_SIZE as u64)
    {
      return Ok(Err(SliceError::OutOfBounds(e)));
    }

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a FixedSliceProvider to exist where self.offset + SELF_SIZE would overflow
        and because offset will for sure be less than or equal to (in the case of a ZST) SELF_SIZE, this too will never overflow.
    */
    self
      .underlying_provider
      .with_read(self.offset + offset, callback)
  }

  fn with_read_dyn<T, CB: for<'a> FnOnce(&'a [u8]) -> T>(
    &self,
    offset: u64,
    size: Option<u64>,
    callback: CB,
  ) -> Result<Result<T, SliceError<Self::StatError>>, ReadError<Self::ReadError>> {
    if let Err(e) = SliceOutOfBoundsError::assert_in_bounds(offset, size, SELF_SIZE as u64) {
      return Ok(Err(SliceError::OutOfBounds(e)));
    }

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a FixedSliceProvider to exist where self.offset + SELF_SIZE would overflow
        and because offset will for sure be less than or equal to (in the case of a ZST) SELF_SIZE, this too will never overflow.
    */
    self
      .underlying_provider
      .with_read_dyn(self.offset + offset, size, callback)
  }

  fn len(&self) -> Result<u64, Self::StatError> {
    Ok(min(
      SELF_SIZE as u64,
      self.underlying_provider.len()? - self.offset,
    ))
  }
}
