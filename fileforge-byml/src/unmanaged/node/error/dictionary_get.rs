use fileforge_lib::{
  error::Error,
  provider::{error::ProviderError, r#trait::Provider, slice::dynamic::DynamicSliceProvider},
  reader::error::underlying_provider_read::UnderlyingProviderReadError,
};

use crate::unmanaged::{
  error::get_string_table::GetStringTableError, string_table::error::get::GetError,
};

use super::get_length::GetLengthError;

pub enum DictionaryGetError<
  'byml_provider,
  'pool,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  GIPE: ProviderError,
  BP: Provider,
> {
  GetStringTableError(GetStringTableError<'pool, BP::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>),
  StringTableGetIndexError(GetError<'pool, GIPE, DIAGNOSTIC_NODE_NAME_SIZE, 128>),
  GetLengthError(
    GetLengthError<
      'pool,
      DynamicSliceProvider<'byml_provider, BP::DynReturnedProviderType>,
      DIAGNOSTIC_NODE_NAME_SIZE,
    >,
  ),
  UnderlyingProviderReadError(
    UnderlyingProviderReadError<'pool, BP::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>,
  ),
}

impl<
    'byml_provider,
    'pool,
    const DIAGNOSTIC_NODE_NAME_SIZE: usize,
    GIPE: ProviderError,
    BP: Provider,
  > Error<DIAGNOSTIC_NODE_NAME_SIZE>
  for DictionaryGetError<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, GIPE, BP>
{
  fn with_report<
    Cb: FnMut(fileforge_lib::error::report::Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> (),
  >(
    &self,
    callback: Cb,
  ) {
    match self {
      Self::GetStringTableError(gste) => gste.with_report(callback),
      Self::StringTableGetIndexError(stgie) => stgie.with_report(callback),
      Self::GetLengthError(gle) => gle.with_report(callback),
      Self::UnderlyingProviderReadError(upre) => upre.with_report(callback),
    }
  }
}
