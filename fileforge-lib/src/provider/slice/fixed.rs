use crate::provider::{
  error::{read_error::ReadError, write_error::WriteError},
  out_of_bounds::SliceOutOfBoundsError,
  r#trait::{MutProvider, Provider},
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

impl<'underlying, const SELF_SIZE: usize, UnderlyingProvider: Provider> Provider
  for FixedSliceProvider<'underlying, SELF_SIZE, UnderlyingProvider>
{
  type ReadError = UnderlyingProvider::ReadError;
  type ReturnedProviderType = UnderlyingProvider;
  type DynReturnedProviderType = UnderlyingProvider;

  fn slice<const SIZE: usize>(
    &self,
    offset: u64,
  ) -> Result<FixedSliceProvider<SIZE, UnderlyingProvider>, SliceOutOfBoundsError> {
    SliceOutOfBoundsError::assert_in_bounds(offset, SIZE as u64, SELF_SIZE as u64)?;

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
    size: u64,
  ) -> Result<DynamicSliceProvider<Self::ReturnedProviderType>, SliceOutOfBoundsError> {
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
  ) -> Result<Result<T, SliceOutOfBoundsError>, ReadError<Self::ReadError>> {
    if let Err(e) = SliceOutOfBoundsError::assert_in_bounds(offset, SIZE as u64, SELF_SIZE as u64) {
      return Ok(Err(e));
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
    size: u64,
    callback: CB,
  ) -> Result<Result<T, SliceOutOfBoundsError>, ReadError<Self::ReadError>> {
    if let Err(e) = SliceOutOfBoundsError::assert_in_bounds(offset, size, SELF_SIZE as u64) {
      return Ok(Err(e));
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

  fn len(&self) -> u64 { SELF_SIZE as u64 }
}

impl<'underlying, const SELF_SIZE: usize, UnderlyingProvider: Provider> Provider
  for FixedMutSliceProvider<'underlying, SELF_SIZE, UnderlyingProvider>
{
  type ReadError = UnderlyingProvider::ReadError;
  type ReturnedProviderType = UnderlyingProvider;
  type DynReturnedProviderType = UnderlyingProvider;

  fn slice<const SIZE: usize>(
    &self,
    offset: u64,
  ) -> Result<FixedSliceProvider<SIZE, UnderlyingProvider>, SliceOutOfBoundsError> {
    SliceOutOfBoundsError::assert_in_bounds(offset, SIZE as u64, SELF_SIZE as u64)?;

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
    size: u64,
  ) -> Result<DynamicSliceProvider<Self::ReturnedProviderType>, SliceOutOfBoundsError> {
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
  ) -> Result<Result<T, SliceOutOfBoundsError>, ReadError<Self::ReadError>> {
    if let Err(e) = SliceOutOfBoundsError::assert_in_bounds(offset, SIZE as u64, SELF_SIZE as u64) {
      return Ok(Err(e));
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
    size: u64,
    callback: CB,
  ) -> Result<Result<T, SliceOutOfBoundsError>, ReadError<Self::ReadError>> {
    if let Err(e) = SliceOutOfBoundsError::assert_in_bounds(offset, size, SELF_SIZE as u64) {
      return Ok(Err(e));
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

  fn len(&self) -> u64 { SELF_SIZE as u64 }
}

impl<'underlying, const SELF_SIZE: usize, UnderlyingProvider: MutProvider> MutProvider
  for FixedMutSliceProvider<'underlying, SELF_SIZE, UnderlyingProvider>
{
  type WriteError = UnderlyingProvider::WriteError;
  type ReturnedMutProviderType = UnderlyingProvider::ReturnedMutProviderType;
  type DynReturnedMutProviderType = UnderlyingProvider::DynReturnedMutProviderType;

  fn slice_mut<const SIZE: usize>(
    &mut self,
    offset: u64,
  ) -> Result<FixedMutSliceProvider<SIZE, UnderlyingProvider>, SliceOutOfBoundsError> {
    SliceOutOfBoundsError::assert_in_bounds(offset, SIZE as u64, SELF_SIZE as u64)?;

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a FixedSliceProvider to exist where self.offset + SELF_SIZE would overflow
        and because offset will for sure be less than or equal to (in the case of a ZST) SELF_SIZE, this too will never overflow.
    */
    Ok(FixedMutSliceProvider {
      underlying_provider: self.underlying_provider,
      offset: self.offset + offset,
    })
  }

  fn slice_mut_dyn(
    &mut self,
    offset: u64,
    size: u64,
  ) -> Result<DynamicMutSliceProvider<Self::ReturnedProviderType>, SliceOutOfBoundsError> {
    SliceOutOfBoundsError::assert_in_bounds(offset, size, SELF_SIZE as u64)?;

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a FixedSliceProvider to exist where self.offset + SELF_SIZE would overflow
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
    if let Err(e) = SliceOutOfBoundsError::assert_in_bounds(offset, SIZE as u64, SELF_SIZE as u64) {
      return Ok(Err(e));
    }

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a FixedSliceProvider to exist where self.offset + SELF_SIZE would overflow
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
    if let Err(e) = SliceOutOfBoundsError::assert_in_bounds(offset, size, SELF_SIZE as u64) {
      return Ok(Err(e));
    }

    /*
     SAFETY:
      - Add will NEVER overflow because it's invalid for a FixedSliceProvider to exist where self.offset + SELF_SIZE would overflow
        and because offset will for sure be less than or equal to (in the case of a ZST) SELF_SIZE, this too will never overflow.
    */
    self
      .underlying_provider
      .with_mut_read_dyn(self.offset + offset, size, callback)
  }

  fn flush(&mut self) -> Result<(), Self::WriteError> { self.underlying_provider.flush() }
}
