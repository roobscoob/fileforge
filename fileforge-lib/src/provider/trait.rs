use super::{error::{read_error::ReadError, write_error::WriteError, ProviderError}, out_of_bounds::SliceOutOfBoundsError, slice::{dynamic::DynamicSliceProvider, fixed::FixedSliceProvider}};

pub trait Provider: Sized {
  type ReadError: ProviderError;
  type WriteError: ProviderError;
  type ReturnedProviderType: Provider<ReadError = Self::ReadError, WriteError = Self::WriteError>;

  // Mutable slice
  fn slice<const SIZE: usize>(&mut self, offset: u64) -> Result<FixedSliceProvider<SIZE, Self::ReturnedProviderType>, SliceOutOfBoundsError>;
  fn slice_dyn(&mut self, offset: u64, size: u64) -> Result<DynamicSliceProvider<Self::ReturnedProviderType>, SliceOutOfBoundsError>;

  // Immutable read
  fn with_read<const SIZE: usize, T, CB: for<'a> FnOnce(&'a [u8; SIZE]) -> T>(&self, offset: u64, callback: CB) -> Result<Result<T, SliceOutOfBoundsError>, ReadError<Self::ReadError>>;
  fn with_read_dyn<T, CB: for<'a> FnOnce(&'a [u8]) -> T>(&self, offset: u64, size: u64, callback: CB) -> Result<Result<T, SliceOutOfBoundsError>, ReadError<Self::ReadError>>;

  // Mutable read
  fn with_mut_read<const SIZE: usize, T, CB: for<'a> FnOnce(&'a mut [u8; SIZE]) -> T>(&mut self, offset: u64, callback: CB) -> Result<Result<T, SliceOutOfBoundsError>, WriteError<Self::WriteError>>;
  fn with_mut_read_dyn<T, CB: for<'a> FnOnce(&'a mut [u8]) -> T>(&mut self, offset: u64, size: u64, callback: CB) -> Result<Result<T, SliceOutOfBoundsError>, WriteError<Self::WriteError>>;

  // Flush
  fn flush(&mut self) -> Result<(), Self::WriteError>;

  // Meta-information
  fn len(&self) -> u64;
}
