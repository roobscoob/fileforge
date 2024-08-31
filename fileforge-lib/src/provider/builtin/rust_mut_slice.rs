use crate::provider::{error::{never::Never, read_error::ReadError, write_error::WriteError}, out_of_bounds::SliceOutOfBoundsError, slice::{dynamic::DynamicSliceProvider, fixed::FixedSliceProvider}, r#trait::Provider};

pub struct RustMutableSliceBinaryProvider<'underlying> {
  underlying_data: &'underlying mut [u8],
}

impl<'underlying> RustMutableSliceBinaryProvider<'underlying> {
  fn slice_internal<const SIZE: usize>(&mut self, offset: u64) -> Result<FixedSliceProvider<SIZE, Self>, SliceOutOfBoundsError> {
    SliceOutOfBoundsError::assert_in_bounds(offset, SIZE as u64, self.underlying_data.len() as u64)?;

    Ok(FixedSliceProvider { underlying_provider: self, offset })
  }
  
  fn slice_dyn_internal(&mut self, offset: u64, size: u64) -> Result<DynamicSliceProvider<Self>, SliceOutOfBoundsError> {
    SliceOutOfBoundsError::assert_in_bounds(offset, size, self.underlying_data.len() as u64)?;

    Ok(DynamicSliceProvider { underlying_provider: self, offset, size })
  }

  fn with_read_internal<const SIZE: usize, T, CB: for<'a> FnOnce(&'a [u8; SIZE]) -> T>(&self, offset: u64, callback: CB) -> Result<T, SliceOutOfBoundsError> {
    let end = SliceOutOfBoundsError::assert_in_bounds(offset, SIZE as u64, self.underlying_data.len() as u64)?;

      /*
        SAFETY:
        - get_unchecked:
          - requires that the bounds be in bound
            this is checked by assert_in_bounds

        - unwrap_unchecked:
          - the range produced by offset..end should always be the length of SIZE
            as assert_in_bounds returns (offset+size) if it didn't overflow.
            this means try_into should always succeed, and thus we can
            unwrap without checks
      */
    Ok(callback(unsafe { self.underlying_data.get_unchecked((offset as usize)..(end as usize)).try_into().unwrap_unchecked() }))
  }

  fn with_read_dyn_internal<T, CB: for<'a> FnOnce(&'a [u8]) -> T>(&self, offset: u64, size: u64, callback: CB) -> Result<T, SliceOutOfBoundsError> {
    let end = SliceOutOfBoundsError::assert_in_bounds(offset, size, self.underlying_data.len() as u64)?;

      /*
        SAFETY:
        - get_unchecked:
          - requires that the bounds be in bound
            this is checked by assert_in_bounds
      */
    Ok(callback(unsafe { self.underlying_data.get_unchecked((offset as usize)..(end as usize)) }))
  }

  fn with_mut_read_internal<const SIZE: usize, T, CB: for<'a> FnOnce(&'a mut [u8; SIZE]) -> T>(&mut self, offset: u64, callback: CB) -> Result<T, SliceOutOfBoundsError> {
    let end = SliceOutOfBoundsError::assert_in_bounds(offset, SIZE as u64, self.underlying_data.len() as u64)?;

      /*
        SAFETY:
        - get_unchecked_mut:
          - requires that the bounds be in bound
            this is checked by assert_in_bounds

        - unwrap_unchecked:
          - the range produced by offset..end should always be the length of SIZE
            as assert_in_bounds returns (offset+size) if it didn't overflow.
            this means try_into should always succeed, and thus we can
            unwrap without checks
      */
    Ok(callback(unsafe { self.underlying_data.get_unchecked_mut((offset as usize)..(end as usize)).try_into().unwrap_unchecked() }))
  }

  fn with_mut_read_dyn_internal<T, CB: for<'a> FnOnce(&'a mut [u8]) -> T>(&mut self, offset: u64, size: u64, callback: CB) -> Result<T, SliceOutOfBoundsError> {
    let end = SliceOutOfBoundsError::assert_in_bounds(offset, size, self.underlying_data.len() as u64)?;

      /*
        SAFETY:
        - get_unchecked_mut:
          - requires that the bounds be in bound
            this is checked by assert_in_bounds
      */
    Ok(callback(unsafe { self.underlying_data.get_unchecked_mut((offset as usize)..(end as usize)) }))
  }
}

impl<'underlying> Provider for RustMutableSliceBinaryProvider<'underlying> {
  type ReadError = Never;
  type WriteError = Never;
  type ReturnedProviderType = Self;
  type DynReturnedProviderType = Self;

  fn slice<const SIZE: usize>(&mut self, offset: u64) -> Result<FixedSliceProvider<SIZE, Self>, SliceOutOfBoundsError>
    { self.slice_internal(offset) }

  fn slice_dyn(&mut self, offset: u64, size: u64) -> Result<DynamicSliceProvider<Self>, SliceOutOfBoundsError>
    { self.slice_dyn_internal(offset, size) }

  fn with_read<const SIZE: usize, T, CB: for<'a> FnOnce(&'a [u8; SIZE]) -> T>(&self, offset: u64, callback: CB) -> Result<Result<T, SliceOutOfBoundsError>, ReadError<Self::ReadError>>
    { Ok(self.with_read_internal(offset, callback)) }

  fn with_read_dyn<T, CB: for<'a> FnOnce(&'a [u8]) -> T>(&self, offset: u64, size: u64, callback: CB) -> Result<Result<T, SliceOutOfBoundsError>, ReadError<Self::ReadError>>
    { Ok(self.with_read_dyn_internal(offset, size, callback)) }

  fn with_mut_read<const SIZE: usize, T, CB: for<'a> FnOnce(&'a mut [u8; SIZE]) -> T>(&mut self, offset: u64, callback: CB) -> Result<Result<T, SliceOutOfBoundsError>, WriteError<Self::WriteError>>
    { Ok(self.with_mut_read_internal(offset, callback)) }

  fn with_mut_read_dyn<T, CB: for<'a> FnOnce(&'a mut [u8]) -> T>(&mut self, offset: u64, size: u64, callback: CB) -> Result<Result<T, SliceOutOfBoundsError>, WriteError<Self::WriteError>>
    { Ok(self.with_mut_read_dyn_internal(offset, size, callback)) }

  fn flush(&mut self) -> Result<(), Self::WriteError>
    { Ok(()) }

  fn len(&self) -> u64
    { self.underlying_data.len() as u64 }
}