use super::{
  error::{read_error::ReadError, slice_error::SliceError, write_error::WriteError, ProviderError},
  out_of_bounds::SliceOutOfBoundsError,
  slice::{
    dynamic::{DynamicMutSliceProvider, DynamicSliceProvider},
    fixed::{FixedMutSliceProvider, FixedSliceProvider},
  },
};

pub trait Provider: Sized {
  type ReadError: ProviderError;
  type StatError: ProviderError;
  type ReturnedProviderType<'underlying, const SIZE: usize>: Provider<
    ReadError = Self::ReadError,
    StatError = Self::StatError,
  >
  where
    Self: 'underlying;
  type DynReturnedProviderType<'underlying>: Provider<
    ReadError = Self::ReadError,
    StatError = Self::StatError,
  >
  where
    Self: 'underlying;

  // Immutable slice
  fn slice<const SIZE: usize>(
    &self,
    offset: u64,
  ) -> Result<Self::ReturnedProviderType<'_, SIZE>, SliceError<Self::StatError>>;
  fn slice_dyn(
    &self,
    offset: u64,
    size: Option<u64>,
  ) -> Result<Self::DynReturnedProviderType<'_>, SliceError<Self::StatError>>;

  // Immutable read
  fn with_read<const SIZE: usize, T, CB: for<'a> FnOnce(&'a [u8; SIZE]) -> T>(
    &self,
    offset: u64,
    callback: CB,
  ) -> Result<Result<T, SliceError<Self::StatError>>, ReadError<Self::ReadError>>;
  fn with_read_dyn<T, CB: for<'a> FnOnce(&'a [u8]) -> T>(
    &self,
    offset: u64,
    size: Option<u64>,
    callback: CB,
  ) -> Result<Result<T, SliceError<Self::StatError>>, ReadError<Self::ReadError>>;

  // Meta-information
  fn len(&self) -> Result<u64, Self::StatError>;
}

pub trait MutProvider: Provider {
  type WriteError: ProviderError;
  type ReturnedMutProviderType: MutProvider<
    ReadError = Self::ReadError,
    WriteError = Self::WriteError,
  >;
  type DynReturnedMutProviderType: MutProvider<
    ReadError = Self::ReadError,
    WriteError = Self::WriteError,
  >;

  // Mutable slice
  fn slice_mut<const SIZE: usize>(
    &mut self,
    offset: u64,
  ) -> Result<
    FixedMutSliceProvider<SIZE, Self::ReturnedProviderType<'_, SIZE>>,
    SliceError<Self::StatError>,
  >;
  fn slice_mut_dyn(
    &mut self,
    offset: u64,
    size: u64,
  ) -> Result<DynamicMutSliceProvider<Self::DynReturnedProviderType<'_>>, SliceError<Self::StatError>>;

  // Mutable read
  fn with_mut_read<const SIZE: usize, T, CB: for<'a> FnOnce(&'a mut [u8; SIZE]) -> T>(
    &mut self,
    offset: u64,
    callback: CB,
  ) -> Result<Result<T, SliceError<Self::StatError>>, WriteError<Self::WriteError>>;
  fn with_mut_read_dyn<T, CB: for<'a> FnOnce(&'a mut [u8]) -> T>(
    &mut self,
    offset: u64,
    size: u64,
    callback: CB,
  ) -> Result<Result<T, SliceError<Self::StatError>>, WriteError<Self::WriteError>>;

  // Flush
  fn flush(&mut self) -> Result<(), Self::WriteError>;
}

impl<P: Provider> Provider for &P {
  type StatError = P::StatError;
  type ReadError = P::ReadError;
  type ReturnedProviderType<'a, const S: usize>
    = P::ReturnedProviderType<'a, S>
  where
    Self: 'a;
  type DynReturnedProviderType<'a>
    = P::DynReturnedProviderType<'a>
  where
    Self: 'a;

  fn slice<const SIZE: usize>(
    &self,
    offset: u64,
  ) -> Result<Self::ReturnedProviderType<'_, SIZE>, SliceError<Self::StatError>> {
    (**self).slice(offset)
  }

  fn slice_dyn(
    &self,
    offset: u64,
    size: Option<u64>,
  ) -> Result<Self::DynReturnedProviderType<'_>, SliceError<Self::StatError>> {
    (**self).slice_dyn(offset, size)
  }

  fn with_read<const SIZE: usize, T, CB: for<'a> FnOnce(&'a [u8; SIZE]) -> T>(
    &self,
    offset: u64,
    callback: CB,
  ) -> Result<Result<T, SliceError<Self::StatError>>, ReadError<Self::ReadError>> {
    (**self).with_read(offset, callback)
  }

  fn with_read_dyn<T, CB: for<'a> FnOnce(&'a [u8]) -> T>(
    &self,
    offset: u64,
    size: Option<u64>,
    callback: CB,
  ) -> Result<Result<T, SliceError<Self::StatError>>, ReadError<Self::ReadError>> {
    (**self).with_read_dyn(offset, size, callback)
  }

  fn len(&self) -> Result<u64, <P as Provider>::StatError> { (**self).len() }
}

impl<P: Provider> Provider for &mut P {
  type StatError = P::StatError;
  type ReadError = P::ReadError;
  type ReturnedProviderType<'a, const S: usize>
    = P::ReturnedProviderType<'a, S>
  where
    Self: 'a;
  type DynReturnedProviderType<'a>
    = P::DynReturnedProviderType<'a>
  where
    Self: 'a;

  fn slice<const SIZE: usize>(
    &self,
    offset: u64,
  ) -> Result<Self::ReturnedProviderType<'_, SIZE>, SliceError<Self::StatError>> {
    (**self).slice(offset)
  }

  fn slice_dyn(
    &self,
    offset: u64,
    size: Option<u64>,
  ) -> Result<Self::DynReturnedProviderType<'_>, SliceError<Self::StatError>> {
    (**self).slice_dyn(offset, size)
  }

  fn with_read<const SIZE: usize, T, CB: for<'a> FnOnce(&'a [u8; SIZE]) -> T>(
    &self,
    offset: u64,
    callback: CB,
  ) -> Result<Result<T, SliceError<Self::StatError>>, ReadError<Self::ReadError>> {
    (**self).with_read(offset, callback)
  }

  fn with_read_dyn<T, CB: for<'a> FnOnce(&'a [u8]) -> T>(
    &self,
    offset: u64,
    size: Option<u64>,
    callback: CB,
  ) -> Result<Result<T, SliceError<Self::StatError>>, ReadError<Self::ReadError>> {
    (**self).with_read_dyn(offset, size, callback)
  }

  fn len(&self) -> Result<u64, <P as Provider>::StatError> { (**self).len() }
}

impl<P: MutProvider> MutProvider for &mut P {
  type WriteError = P::WriteError;
  type ReturnedMutProviderType = P::ReturnedMutProviderType;
  type DynReturnedMutProviderType = P::DynReturnedMutProviderType;

  fn slice_mut<const SIZE: usize>(
    &mut self,
    offset: u64,
  ) -> Result<
    FixedMutSliceProvider<SIZE, Self::ReturnedProviderType<'_, SIZE>>,
    SliceError<Self::StatError>,
  > {
    (**self).slice_mut(offset)
  }

  fn slice_mut_dyn(
    &mut self,
    offset: u64,
    size: u64,
  ) -> Result<DynamicMutSliceProvider<Self::DynReturnedProviderType<'_>>, SliceError<Self::StatError>>
  {
    (**self).slice_mut_dyn(offset, size)
  }

  fn with_mut_read<const SIZE: usize, T, CB: for<'a> FnOnce(&'a mut [u8; SIZE]) -> T>(
    &mut self,
    offset: u64,
    callback: CB,
  ) -> Result<Result<T, SliceError<Self::StatError>>, WriteError<Self::WriteError>> {
    (**self).with_mut_read(offset, callback)
  }

  fn with_mut_read_dyn<T, CB: for<'a> FnOnce(&'a mut [u8]) -> T>(
    &mut self,
    offset: u64,
    size: u64,
    callback: CB,
  ) -> Result<Result<T, SliceError<Self::StatError>>, WriteError<Self::WriteError>> {
    (**self).with_mut_read_dyn(offset, size, callback)
  }

  fn flush(&mut self) -> Result<(), Self::WriteError> { (**self).flush() }
}
