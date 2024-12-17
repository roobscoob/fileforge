use fileforge_lib::provider::{error::ProviderError, r#trait::Provider};

use crate::unmanaged::{
  error::get_string_table::GetStringTableError, string_table::error::get::GetError,
};

use super::get_length::GetLengthError;

pub enum DictionaryGetError<
  'pool,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  RP: Provider,
  GIPE: ProviderError,
  BP: Provider,
> {
  GetStringTableError(GetStringTableError<'pool, BP::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>),
  StringTableGetIndexError(GetError<'pool, GIPE, DIAGNOSTIC_NODE_NAME_SIZE, 128>),
  GetLengthError(GetLengthError<'pool, RP, DIAGNOSTIC_NODE_NAME_SIZE>),
  EntryOutOfBounds(),
}
