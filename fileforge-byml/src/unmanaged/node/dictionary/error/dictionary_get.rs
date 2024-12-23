use fileforge_lib::{
  error::Error,
  provider::{error::ProviderError, r#trait::Provider, slice::dynamic::DynamicSliceProvider},
  reader::error::underlying_provider_error::UnderlyingProviderError,
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
  GISE: ProviderError,
  BP: Provider + 'byml_provider,
> {
  GetStringTableError(
    GetStringTableError<'pool, BP::ReadError, BP::StatError, DIAGNOSTIC_NODE_NAME_SIZE>,
  ),
  StringTableGetIndexError(GetError<'pool, GIPE, GISE, DIAGNOSTIC_NODE_NAME_SIZE, 128>),
  GetLengthError(
    GetLengthError<
      'pool,
      DynamicSliceProvider<'byml_provider, BP::DynReturnedProviderType<'byml_provider>>,
      DIAGNOSTIC_NODE_NAME_SIZE,
    >,
  ),
  UnderlyingProviderError(
    UnderlyingProviderError<'pool, BP::ReadError, BP::StatError, DIAGNOSTIC_NODE_NAME_SIZE>,
  ),
}

impl<
    'byml_provider,
    'pool,
    const DIAGNOSTIC_NODE_NAME_SIZE: usize,
    GIPE: ProviderError,
    GISE: ProviderError,
    BP: Provider,
  > Error<DIAGNOSTIC_NODE_NAME_SIZE>
  for DictionaryGetError<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, GIPE, GISE, BP>
{
  fn with_report<
    Cb: FnMut(fileforge_lib::error::report::Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> (),
  >(
    &self,
    callback: Cb,
  ) {
    match self {
      Self::GetStringTableError(gste) => Error::with_report(gste, callback),
      Self::StringTableGetIndexError(stgie) => Error::with_report(stgie, callback),
      Self::GetLengthError(gle) => Error::with_report(gle, callback),
      Self::UnderlyingProviderError(upre) => Error::with_report(upre, callback),
    }
  }
}
