use core::any::{type_name, type_name_of_val};
use std::{
  backtrace::{Backtrace, BacktraceStatus},
  fs::File,
  io::Read,
  os::windows::fs::FileExt,
  path::Path,
  println, vec,
};

use fileforge_lib::provider::{
  error::{read_error::ReadError, slice_error::SliceError, ProviderError},
  out_of_bounds::SliceOutOfBoundsError,
  r#trait::Provider,
  slice::{dynamic::DynamicSliceProvider, fixed::FixedSliceProvider},
};

pub struct LogProvider<UnderlyingProvider: Provider> {
  underlying_provider: UnderlyingProvider,
}

impl<UnderlyingProvider: Provider> LogProvider<UnderlyingProvider> {
  pub fn over(other: UnderlyingProvider) -> Self {
    Self {
      underlying_provider: other,
    }
  }
}

impl<UnderlyingProvider: Provider> Provider for LogProvider<UnderlyingProvider> {
  type ReadError = UnderlyingProvider::ReadError;
  type StatError = UnderlyingProvider::StatError;
  type DynReturnedProviderType<'l>
    = LogProvider<UnderlyingProvider::DynReturnedProviderType<'l>>
  where
    UnderlyingProvider: 'l;
  type ReturnedProviderType<'l, const SIZE: usize>
    = LogProvider<UnderlyingProvider::ReturnedProviderType<'l, SIZE>>
  where
    UnderlyingProvider: 'l;

  fn len(&self) -> Result<u64, Self::StatError> {
    let bt = Backtrace::capture();
    match self.underlying_provider.len() {
      Ok(v) => {
        if Backtrace::status(&bt) == BacktraceStatus::Captured {
          println!(
            "LogProvider<{}>::len() -> Ok({})\n{}",
            type_name::<UnderlyingProvider>(),
            v,
            bt
          );
        } else {
          println!(
            "LogProvider<{}>::len() -> Ok({})",
            type_name::<UnderlyingProvider>(),
            v
          );
        }
        Ok(v)
      }
      Err(e) => {
        if Backtrace::status(&bt) == BacktraceStatus::Captured {
          println!(
            "LogProvider<{}>::len() -> Err(...)\n{}",
            type_name::<UnderlyingProvider>(),
            bt
          );
        } else {
          println!(
            "LogProvider<{}>::len() -> Err(...)",
            type_name::<UnderlyingProvider>()
          );
        }
        Err(e)
      }
    }
  }

  fn slice<const SIZE: usize>(
    &self,
    offset: u64,
  ) -> Result<Self::ReturnedProviderType<'_, SIZE>, SliceError<Self::StatError>> {
    let bt = Backtrace::capture();
    match self.underlying_provider.slice(offset) {
      Ok(v) => {
        if Backtrace::status(&bt) == BacktraceStatus::Captured {
          println!(
            "LogProvider<{}>::slice<SIZE = {}>(offset: {}) -> Ok({})\n{}",
            type_name::<UnderlyingProvider>(),
            SIZE,
            offset,
            type_name_of_val(&v),
            bt
          );
        } else {
          println!(
            "LogProvider<{}>::slice<SIZE = {}>(offset: {}) -> Ok({})",
            type_name::<UnderlyingProvider>(),
            SIZE,
            offset,
            type_name_of_val(&v)
          );
        }
        Ok(LogProvider::over(v))
      }
      Err(e) => {
        if Backtrace::status(&bt) == BacktraceStatus::Captured {
          println!(
            "LogProvider<{}>::slice<SIZE = {}>(offset: {}) -> Err(...)\n{}",
            type_name::<UnderlyingProvider>(),
            SIZE,
            offset,
            bt
          );
        } else {
          println!(
            "LogProvider<{}>::slice<SIZE = {}>(offset: {}) -> Err(...)",
            type_name::<UnderlyingProvider>(),
            SIZE,
            offset
          );
        }
        Err(e)
      }
    }
  }

  fn slice_dyn(
    &self,
    offset: u64,
    size: Option<u64>,
  ) -> Result<Self::DynReturnedProviderType<'_>, SliceError<Self::StatError>> {
    let bt = Backtrace::capture();
    match self.underlying_provider.slice_dyn(offset, size) {
      Ok(v) => {
        if Backtrace::status(&bt) == BacktraceStatus::Captured {
          println!(
            "LogProvider<{}>::slice_dyn(offset: {}, size: {:?}) -> Ok({})\n{}",
            type_name::<UnderlyingProvider>(),
            offset,
            size,
            type_name_of_val(&v),
            bt
          );
        } else {
          println!(
            "LogProvider<{}>::slice_dyn(offset: {}, size: {:?}) -> Ok({})",
            type_name::<UnderlyingProvider>(),
            offset,
            size,
            type_name_of_val(&v)
          );
        }
        Ok(LogProvider::over(v))
      }
      Err(e) => {
        if Backtrace::status(&bt) == BacktraceStatus::Captured {
          println!(
            "LogProvider<{}>::slice_dyn(offset: {}, size: {:?}) -> Err(...)\n{}",
            type_name::<UnderlyingProvider>(),
            offset,
            size,
            bt
          );
        } else {
          println!(
            "LogProvider<{}>::slice_dyn(offset: {}, size: {:?}) -> Err(...)",
            type_name::<UnderlyingProvider>(),
            offset,
            size
          );
        }
        Err(e)
      }
    }
  }

  fn with_read<const SIZE: usize, T, CB: for<'a> FnOnce(&'a [u8; SIZE]) -> T>(
    &self,
    offset: u64,
    callback: CB,
  ) -> Result<Result<T, SliceError<Self::StatError>>, ReadError<Self::ReadError>> {
    let bt = Backtrace::capture();
    match self.underlying_provider.with_read(offset, |v| {
      if Backtrace::status(&bt) == BacktraceStatus::Captured {
        println!(
          "LogProvider<{}>::with_read::<SIZE = {}>(offset: {}) -> Ok(Ok(...))\n{}",
          type_name::<UnderlyingProvider>(),
          SIZE,
          offset,
          bt
        );
      } else {
        println!(
          "LogProvider<{}>::with_read::<SIZE = {}>(offset: {}) -> Ok(Ok(...))",
          type_name::<UnderlyingProvider>(),
          SIZE,
          offset
        );
      }
      callback(v)
    }) {
      Ok(Ok(v)) => Ok(Ok(v)),
      Ok(Err(SliceError::OutOfBounds(oob))) => {
        if Backtrace::status(&bt) == BacktraceStatus::Captured {
          println!(
            "LogProvider<{}>::with_read::<SIZE = {}>(offset: {}) -> Ok(Err(SliceError::OutOfBounds({:?})))\n{}",
            type_name::<UnderlyingProvider>(),
            SIZE,
            offset,
            oob,
            bt
          );
        } else {
          println!(
            "LogProvider<{}>::with_read::<SIZE = {}>(offset: {}) -> Ok(Err(SliceError::OutOfBounds({:?}))",
            type_name::<UnderlyingProvider>(),
            SIZE,
            offset,
            oob
          );
        }
        Ok(Err(SliceError::OutOfBounds(oob)))
      }
      Ok(Err(SliceError::StatError(e))) => {
        if Backtrace::status(&bt) == BacktraceStatus::Captured {
          println!(
            "LogProvider<{}>::with_read::<SIZE = {}>(offset: {}) -> Ok(Err(SliceError::StatError(...)))\n{}",
            type_name::<UnderlyingProvider>(),
            SIZE,
            offset,
            bt
          );
        } else {
          println!(
            "LogProvider<{}>::with_read::<SIZE = {}>(offset: {}) -> Ok(Err(SliceError::StatError(...)))",
            type_name::<UnderlyingProvider>(),
            SIZE,
            offset
          );
        }
        Ok(Err(SliceError::StatError(e)))
      }
      Err(e) => {
        if Backtrace::status(&bt) == BacktraceStatus::Captured {
          println!(
            "LogProvider<{}>::with_read::<SIZE = {}>(offset: {}) -> Err(...)\n{}",
            type_name::<UnderlyingProvider>(),
            SIZE,
            offset,
            bt
          );
        } else {
          println!(
            "LogProvider<{}>::with_read::<SIZE = {}>(offset: {}) -> Err(...)",
            type_name::<UnderlyingProvider>(),
            SIZE,
            offset
          );
        }
        Err(e)
      }
    }
  }

  fn with_read_dyn<T, CB: for<'a> FnOnce(&'a [u8]) -> T>(
    &self,
    offset: u64,
    size: Option<u64>,
    callback: CB,
  ) -> Result<Result<T, SliceError<Self::StatError>>, ReadError<Self::ReadError>> {
    let bt = Backtrace::capture();
    match self.underlying_provider.with_read_dyn(offset, size, |v| {
      if Backtrace::status(&bt) == BacktraceStatus::Captured {
        println!(
          "LogProvider<{}>::with_read(offset: {}, size: {:?}) -> Ok(Ok(...))\n{}",
          type_name::<UnderlyingProvider>(),
          offset,
          size,
          bt,
        );
      } else {
        println!(
          "LogProvider<{}>::with_read(offset: {}, size: {:?}) -> Ok(Ok(...))",
          type_name::<UnderlyingProvider>(),
          offset,
          size,
        );
      }
      callback(v)
    }) {
      Ok(Ok(v)) => Ok(Ok(v)),
      Ok(Err(SliceError::OutOfBounds(oob))) => {
        if Backtrace::status(&bt) == BacktraceStatus::Captured {
          println!(
            "LogProvider<{}>::with_read(offset: {}, size: {:?}) -> Ok(Err(SliceError::OutOfBounds({:?})))\n{}",
            type_name::<UnderlyingProvider>(),
            offset,
            size,
            oob,
            bt
          );
        } else {
          println!(
            "LogProvider<{}>::with_read(offset: {}, size: {:?}) -> Ok(Err(SliceError::OutOfBounds({:?})))",
            type_name::<UnderlyingProvider>(),
            offset,
            size,
            oob
          );
        }
        Ok(Err(SliceError::OutOfBounds(oob)))
      }
      Ok(Err(SliceError::StatError(e))) => {
        if Backtrace::status(&bt) == BacktraceStatus::Captured {
          println!(
            "LogProvider<{}>::with_read(offset: {}, size: {:?}) -> Ok(Err(SliceError::StatError(...)))\n{}",
            type_name::<UnderlyingProvider>(),
            offset,
            size,
            bt
          );
        } else {
          println!(
            "LogProvider<{}>::with_read(offset: {}, size: {:?}) -> Ok(Err(SliceError::StatError(...)))",
            type_name::<UnderlyingProvider>(),
            offset,
            size
          );
        }
        Ok(Err(SliceError::StatError(e)))
      }
      Err(e) => {
        if Backtrace::status(&bt) == BacktraceStatus::Captured {
          println!(
            "LogProvider<{}>::with_read(offset: {}, size: {:?}) -> Err(...)\n{}",
            type_name::<UnderlyingProvider>(),
            offset,
            size,
            bt
          );
        } else {
          println!(
            "LogProvider<{}>::with_read(offset: {}, size: {:?}) -> Err(...)",
            type_name::<UnderlyingProvider>(),
            offset,
            size
          );
        }
        Err(e)
      }
    }
  }
}
