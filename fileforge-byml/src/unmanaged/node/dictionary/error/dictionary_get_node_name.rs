use fileforge_lib::{error::Error, provider::r#trait::Provider};

use crate::unmanaged::{
  error::get_string_table::GetStringTableError, string_table::error::get::GetError,
};

pub enum DictionaryGetNodeNameError<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, BP: Provider> {
  GetStringTableError(
    GetStringTableError<'pool, BP::ReadError, BP::StatError, DIAGNOSTIC_NODE_NAME_SIZE>,
  ),
  GetStringTableEntryError(
    GetError<'pool, BP::ReadError, BP::StatError, DIAGNOSTIC_NODE_NAME_SIZE, 128>,
  ),
  NoStringTable,
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, BP: Provider> Error<DIAGNOSTIC_NODE_NAME_SIZE>
  for DictionaryGetNodeNameError<'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
{
  fn with_report<
    Cb: FnMut(fileforge_lib::error::report::Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> (),
  >(
    &self,
    callback: Cb,
  ) {
    match self {
      Self::GetStringTableEntryError(e) => Error::with_report(e, callback),
      Self::GetStringTableError(e) => Error::with_report(e, callback),
      Self::NoStringTable => unimplemented!(),
    }
  }
}
