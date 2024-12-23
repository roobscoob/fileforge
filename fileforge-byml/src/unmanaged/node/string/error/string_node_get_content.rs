use fileforge_lib::{
  error::Error,
  provider::{
    error::{never::Never, ProviderError},
    r#trait::Provider,
    slice::dynamic::DynamicSliceProvider,
  },
  reader::error::underlying_provider_read::UnderlyingProviderReadError,
};

use crate::unmanaged::{
  error::get_string_table::GetStringTableError, string_table::error::get::GetError,
};

pub enum StringNodeGetContent<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, BP: Provider> {
  GetStringTableError(
    GetStringTableError<'pool, BP::ReadError, BP::StatError, DIAGNOSTIC_NODE_NAME_SIZE>,
  ),
  GetStringTableEntryError(
    GetError<'pool, BP::ReadError, BP::StatError, DIAGNOSTIC_NODE_NAME_SIZE, 128>,
  ),
  NoStringTable,
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, BP: Provider> Error<DIAGNOSTIC_NODE_NAME_SIZE>
  for StringNodeGetContent<'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
{
  fn with_report<
    Cb: FnMut(fileforge_lib::error::report::Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> (),
  >(
    &self,
    callback: Cb,
  ) {
    match self {
      Self::GetStringTableEntryError(e) => e.with_report(callback),
      Self::GetStringTableError(e) => e.with_report(callback),
      Self::NoStringTable => todo!(),
    }
  }
}
