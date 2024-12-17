use crate::provider::{
  error::{read_error::ReadError, write_error::WriteError},
  out_of_bounds::SliceOutOfBoundsError,
  r#trait::{MutProvider, Provider},
};

use super::fixed::{FixedMutSliceProvider, FixedSliceProvider};

pub struct DynamicMutSliceProvider<'a, T: Provider> {
  pub(crate) underlying_provider: &'a mut T,
  pub(crate) offset: u64,
  pub(crate) size: u64,
}

#[derive(Clone)]
pub struct DynamicSliceProvider<'a, T: Provider> {
  pub(crate) underlying_provider: &'a T,
  pub(crate) offset: u64,
  pub(crate) size: u64,
}

impl<'underlying, UnderlyingProvider: Provider> Provider
  for DynamicSliceProvider<'underlying, UnderlyingProvider>
{
  type ReadError = UnderlyingProvider::ReadError;
  type ReturnedProviderType = UnderlyingProvider;
  type DynReturnedProviderType = UnderlyingProvider;

  fn slice<const SIZE: usize>(
    &self,
    offset: u64,
  ) -> Result<FixedSliceProvider<SIZE, UnderlyingProvider>, SliceOutOfBoundsError> {
    SliceOutOfBoundsError::assert_in_bounds(offset, SIZE as u64, self.size)?;

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
    size: u64,
  ) -> Result<DynamicSliceProvider<'x, Self::ReturnedProviderType>, SliceOutOfBoundsError> {
    SliceOutOfBoundsError::assert_in_bounds(offset, size, self.size)?;

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
  ) -> Result<Result<T, SliceOutOfBoundsError>, ReadError<Self::ReadError>> {
    if let Err(e) = SliceOutOfBoundsError::assert_in_bounds(offset, SIZE as u64, self.size) {
      return Ok(Err(e));
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
    size: u64,
    callback: CB,
  ) -> Result<Result<T, SliceOutOfBoundsError>, ReadError<Self::ReadError>> {
    if let Err(e) = SliceOutOfBoundsError::assert_in_bounds(offset, size, self.size) {
      return Ok(Err(e));
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

  fn len(&self) -> u64 { self.size }
}

impl<'underlying, UnderlyingProvider: Provider> Provider
  for DynamicMutSliceProvider<'underlying, UnderlyingProvider>
{
  type ReadError = UnderlyingProvider::ReadError;
  type ReturnedProviderType = UnderlyingProvider;
  type DynReturnedProviderType = UnderlyingProvider;

  fn slice<const SIZE: usize>(
    &self,
    offset: u64,
  ) -> Result<FixedSliceProvider<SIZE, UnderlyingProvider>, SliceOutOfBoundsError> {
    SliceOutOfBoundsError::assert_in_bounds(offset, SIZE as u64, self.size)?;

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
    size: u64,
  ) -> Result<DynamicSliceProvider<'x, Self::ReturnedProviderType>, SliceOutOfBoundsError> {
    SliceOutOfBoundsError::assert_in_bounds(offset, size, self.size)?;

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
  ) -> Result<Result<T, SliceOutOfBoundsError>, ReadError<Self::ReadError>> {
    if let Err(e) = SliceOutOfBoundsError::assert_in_bounds(offset, SIZE as u64, self.size) {
      return Ok(Err(e));
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
    size: u64,
    callback: CB,
  ) -> Result<Result<T, SliceOutOfBoundsError>, ReadError<Self::ReadError>> {
    if let Err(e) = SliceOutOfBoundsError::assert_in_bounds(offset, size, self.size) {
      return Ok(Err(e));
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

  fn len(&self) -> u64 { self.size }
}

impl<'underlying, UnderlyingProvider: MutProvider> MutProvider
  for DynamicMutSliceProvider<'underlying, UnderlyingProvider>
{
  type WriteError = UnderlyingProvider::WriteError;
  type ReturnedMutProviderType = UnderlyingProvider;
  type DynReturnedMutProviderType = UnderlyingProvider;

  fn slice_mut<const SIZE: usize>(
    &mut self,
    offset: u64,
  ) -> Result<FixedMutSliceProvider<SIZE, UnderlyingProvider>, SliceOutOfBoundsError> {
    SliceOutOfBoundsError::assert_in_bounds(offset, SIZE as u64, self.size)?;

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a DynamicSliceProvider to exist where self.offset + self.size would overflow
        and because offset will for sure be less than or equal to (in the case of a ZST) SELF_SIZE, this too will never overflow.
    */
    Ok(FixedMutSliceProvider {
      underlying_provider: self.underlying_provider,
      offset: self.offset + offset,
    })
  }

  fn slice_mut_dyn<'x>(
    &'x mut self,
    offset: u64,
    size: u64,
  ) -> Result<DynamicMutSliceProvider<'x, Self::ReturnedProviderType>, SliceOutOfBoundsError> {
    SliceOutOfBoundsError::assert_in_bounds(offset, size, self.size)?;

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a DynamicSliceProvider to exist where self.offset + self.size would overflow
        and because offset will for sure be less than or equal to (in the case of a ZST) SELF_SIZE, this too will never overflow.
    */
    Ok(DynamicMutSliceProvider {
      underlying_provider: self.underlying_provider,
      offset: self.offset + offset,
      size,
    })
  }

  fn with_mut_read<const SIZE: usize, T, CB: for<'a> FnOnce(&'a mut [u8; SIZE]) -> T>(
    &mut self,
    offset: u64,
    callback: CB,
  ) -> Result<Result<T, SliceOutOfBoundsError>, WriteError<Self::WriteError>> {
    if let Err(e) = SliceOutOfBoundsError::assert_in_bounds(offset, SIZE as u64, self.size) {
      return Ok(Err(e));
    }

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a DynamicSliceProvider to exist where self.offset + self.size would overflow
        and because offset will for sure be less than or equal to (in the case of a ZST) SELF_SIZE, this too will never overflow.
    */
    self
      .underlying_provider
      .with_mut_read(self.offset + offset, callback)
  }

  fn with_mut_read_dyn<T, CB: for<'a> FnOnce(&'a mut [u8]) -> T>(
    &mut self,
    offset: u64,
    size: u64,
    callback: CB,
  ) -> Result<Result<T, SliceOutOfBoundsError>, WriteError<Self::WriteError>> {
    if let Err(e) = SliceOutOfBoundsError::assert_in_bounds(offset, size, self.size) {
      return Ok(Err(e));
    }

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a DynamicSliceProvider to exist where self.offset + self.size would overflow
        and because offset will for sure be less than or equal to (in the case of a ZST) SELF_SIZE, this too will never overflow.
    */
    self
      .underlying_provider
      .with_mut_read_dyn(self.offset + offset, size, callback)
  }

  fn flush(&mut self) -> Result<(), Self::WriteError> { self.underlying_provider.flush() }
}
