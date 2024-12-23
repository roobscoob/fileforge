use std::{fs::File, io::Read, os::windows::fs::FileExt, path::Path, vec};

use fileforge_lib::{
  provider::{
    error::{read_error::ReadError, slice_error::SliceError, ProviderError},
    out_of_bounds::SliceOutOfBoundsError,
    r#trait::Provider,
    slice::{dynamic::DynamicSliceProvider, fixed::FixedSliceProvider},
  },
  reader::error::underlying_provider_stat::UnderlyingProviderStatError,
};

pub struct FileProvider {
  file: File,
}

impl FileProvider {
  pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<FileProvider> {
    Ok(FileProvider {
      file: File::open(path)?,
    })
  }
}

impl Provider for FileProvider {
  type ReadError = IoError;
  type StatError = IoError;
  type DynReturnedProviderType<'l> = DynamicSliceProvider<'l, Self>;
  type ReturnedProviderType<'l, const SIZE: usize> = FixedSliceProvider<'l, SIZE, Self>;

  fn len(&self) -> Result<u64, IoError> { Ok(self.file.metadata()?.len()) }

  fn slice<const SIZE: usize>(
    &self,
    offset: u64,
  ) -> Result<Self::ReturnedProviderType<'_, SIZE>, SliceError<Self::StatError>> {
    FixedSliceProvider::over(self, offset)
  }

  fn slice_dyn(
    &self,
    offset: u64,
    size: Option<u64>,
  ) -> Result<Self::DynReturnedProviderType<'_>, SliceError<Self::StatError>> {
    DynamicSliceProvider::over(self, offset, size)
  }

  fn with_read<const SIZE: usize, T, CB: for<'a> FnOnce(&'a [u8; SIZE]) -> T>(
    &self,
    offset: u64,
    callback: CB,
  ) -> Result<Result<T, SliceError<Self::StatError>>, ReadError<Self::ReadError>> {
    let mut data = [0; SIZE];

    let size = self
      .file
      .seek_read(&mut data, offset)
      .map_err(|e| ReadError(e.into()))?;

    if size < SIZE {
      return Ok(Err(SliceError::OutOfBounds(SliceOutOfBoundsError {
        read_offset: offset,
        read_size: Some(SIZE as u64),
        provider_size: offset + size as u64,
      })));
    }

    Ok(Ok(callback(&data)))
  }

  fn with_read_dyn<T, CB: for<'a> FnOnce(&'a [u8]) -> T>(
    &self,
    offset: u64,
    size: Option<u64>,
    callback: CB,
  ) -> Result<Result<T, SliceError<Self::StatError>>, ReadError<Self::ReadError>> {
    let size = match size {
      Some(v) => v,
      None => match self.len() {
        Ok(v) => v - offset,
        Err(e) => return Ok(Err(SliceError::StatError(UnderlyingProviderStatError(e)))),
      },
    };

    let mut data = vec![0; size as usize];

    let size2 = self
      .file
      .seek_read(&mut data, offset)
      .map_err(|e| ReadError(e.into()))?;

    if (size2 as u64) < size {
      return Ok(Err(SliceError::OutOfBounds(SliceOutOfBoundsError {
        read_offset: offset,
        read_size: Some(size2 as u64),
        provider_size: offset + size as u64,
      })));
    }

    Ok(Ok(callback(&data)))
  }
}

#[derive(Debug)]
pub struct IoError(std::io::Error);

impl From<std::io::Error> for IoError {
  fn from(value: std::io::Error) -> Self { IoError(value) }
}

impl ProviderError for IoError {
  fn with_report<
    'pool,
    Cb: FnMut(fileforge_lib::error::report::Report<NODE_NAME_SIZE>) -> (),
    const NODE_NAME_SIZE: usize,
  >(
    &self,
    location: Option<
      fileforge_lib::diagnostic::node::reference::DiagnosticReference<'pool, NODE_NAME_SIZE>,
    >,
    callback: Cb,
  ) {
    panic!()
  }
}
