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

use super::fixed::{FixedMutSliceProvider, FixedSliceProvider};

pub struct DynamicMutSliceProvider<'a, T: Provider> {
  pub(crate) underlying_provider: &'a mut T,
  pub(crate) offset: u64,
  pub(crate) size: Option<u64>,
}

pub struct DynamicSliceProvider<'a, T: Provider> {
  pub(crate) underlying_provider: &'a T,
  pub(crate) offset: u64,
  pub(crate) size: Option<u64>,
}

impl<'underlying, T: Provider> DynamicSliceProvider<'underlying, T> {
  pub fn over(
    underlying: &'underlying T,
    offset: u64,
    size: Option<u64>,
  ) -> Result<DynamicSliceProvider<'underlying, T>, SliceError<T::StatError>> {
    SliceOutOfBoundsError::assert_in_bounds(
      offset,
      size,
      underlying
        .len()
        .map_err(|e| UnderlyingProviderStatError(e))?,
    )?;

    Ok(Self {
      offset,
      size,
      underlying_provider: underlying,
    })
  }
}

impl<'a, T: Provider> Clone for DynamicSliceProvider<'a, T> {
  fn clone(&self) -> Self {
    Self {
      underlying_provider: self.underlying_provider,
      offset: self.offset,
      size: self.size,
    }
  }
}

impl<'underlying, UnderlyingProvider: Provider> Provider
  for DynamicSliceProvider<'underlying, UnderlyingProvider>
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
    SliceOutOfBoundsError::assert_in_bounds(
      offset,
      Some(SIZE as u64),
      self
        .len()
        .map_err(|e| SliceError::StatError(UnderlyingProviderStatError(e)))?,
    )?;

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a DynamicSliceProvider to exist where self.offset + self.size would overflow
        and because offset will for sure be less than or equal to (in the case of a ZST) SELF_SIZE, this too will never overflow.
    */
    Ok(FixedSliceProvider {
      underlying_provider: self.underlying_provider,
      offset: self.offset + offset,
    })
  }

  fn slice_dyn<'x>(
    &'x self,
    offset: u64,
    size: Option<u64>,
  ) -> Result<Self::DynReturnedProviderType<'x>, SliceError<Self::StatError>> {
    SliceOutOfBoundsError::assert_in_bounds(
      offset,
      size,
      self
        .len()
        .map_err(|e| SliceError::StatError(UnderlyingProviderStatError(e)))?,
    )?;

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a DynamicSliceProvider to exist where self.offset + self.size would overflow
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
    if let Err(e) = SliceOutOfBoundsError::assert_in_bounds(
      offset,
      Some(SIZE as u64),
      match self
        .len()
        .map_err(|e| SliceError::StatError(UnderlyingProviderStatError(e)))
      {
        Ok(v) => v,
        Err(e) => return Ok(Err(e)),
      },
    ) {
      return Ok(Err(SliceError::OutOfBounds(e)));
    }

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a DynamicSliceProvider to exist where self.offset + self.size would overflow
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
    if let Err(e) = SliceOutOfBoundsError::assert_in_bounds(
      offset,
      size,
      match self
        .len()
        .map_err(|e| SliceError::StatError(UnderlyingProviderStatError(e)))
      {
        Ok(v) => v,
        Err(e) => return Ok(Err(e)),
      },
    ) {
      return Ok(Err(SliceError::OutOfBounds(e)));
    }

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a DynamicSliceProvider to exist where self.offset + self.size would overflow
        and because offset will for sure be less than or equal to (in the case of a ZST) SELF_SIZE, this too will never overflow.
    */
    self
      .underlying_provider
      .with_read_dyn(self.offset + offset, size, callback)
  }

  fn len(&self) -> Result<u64, Self::StatError> {
    let underlying_len = self.underlying_provider.len()?;

    Ok(
      self
        .size
        .map(|v| min(v, underlying_len - self.offset))
        .unwrap_or(underlying_len - self.offset),
    )
  }
}

impl<'underlying, UnderlyingProvider: Provider> Provider
  for DynamicMutSliceProvider<'underlying, UnderlyingProvider>
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
    SliceOutOfBoundsError::assert_in_bounds(
      offset,
      Some(SIZE as u64),
      self
        .len()
        .map_err(|e| SliceError::StatError(UnderlyingProviderStatError(e)))?,
    )?;

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a DynamicSliceProvider to exist where self.offset + self.size would overflow
        and because offset will for sure be less than or equal to (in the case of a ZST) SELF_SIZE, this too will never overflow.
    */
    Ok(FixedSliceProvider {
      underlying_provider: self.underlying_provider,
      offset: self.offset + offset,
    })
  }

  fn slice_dyn<'x>(
    &'x self,
    offset: u64,
    size: Option<u64>,
  ) -> Result<Self::DynReturnedProviderType<'x>, SliceError<Self::StatError>> {
    SliceOutOfBoundsError::assert_in_bounds(
      offset,
      size,
      self
        .len()
        .map_err(|e| SliceError::StatError(UnderlyingProviderStatError(e)))?,
    )?;

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a DynamicSliceProvider to exist where self.offset + self.size would overflow
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
    if let Err(e) = SliceOutOfBoundsError::assert_in_bounds(
      offset,
      Some(SIZE as u64),
      match self
        .len()
        .map_err(|e| SliceError::StatError(UnderlyingProviderStatError(e)))
      {
        Ok(v) => v,
        Err(e) => return Ok(Err(e)),
      },
    ) {
      return Ok(Err(SliceError::OutOfBounds(e)));
    }

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a DynamicSliceProvider to exist where self.offset + self.size would overflow
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
    if let Err(e) = SliceOutOfBoundsError::assert_in_bounds(
      offset,
      size,
      match self
        .len()
        .map_err(|e| SliceError::StatError(UnderlyingProviderStatError(e)))
      {
        Ok(v) => v,
        Err(e) => return Ok(Err(e)),
      },
    ) {
      return Ok(Err(SliceError::OutOfBounds(e)));
    }

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a DynamicSliceProvider to exist where self.offset + self.size would overflow
        and because offset will for sure be less than or equal to (in the case of a ZST) SELF_SIZE, this too will never overflow.
    */
    self
      .underlying_provider
      .with_read_dyn(self.offset + offset, size, callback)
  }

  fn len(&self) -> Result<u64, Self::StatError> {
    let underlying_len = self.underlying_provider.len()?;

    Ok(
      self
        .size
        .map(|v| min(v, underlying_len - self.offset))
        .unwrap_or(underlying_len - self.offset),
    )
  }
}
