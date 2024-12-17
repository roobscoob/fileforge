use fileforge_lib::{
  error::{report::Report, Error},
  provider::error::ProviderError,
  reader::error::{
    expect_primitive::ExpectationFailedError, underlying_provider_read::UnderlyingProviderReadError,
  },
};

use crate::unmanaged::error::get_string_table::StringTableOutOfBounds;

pub enum StringTableSizeError<'pool, Pe: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  UnderlyingProviderReadError(UnderlyingProviderReadError<'pool, Pe, DIAGNOSTIC_NODE_NAME_SIZE>),

  InvalidNodeKind(ExpectationFailedError<'pool, u8, 1, DIAGNOSTIC_NODE_NAME_SIZE>),
  NotLargeEnough(StringTableOutOfBounds<'pool, DIAGNOSTIC_NODE_NAME_SIZE>),
}

impl<'pool, Pe: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize>
  From<UnderlyingProviderReadError<'pool, Pe, DIAGNOSTIC_NODE_NAME_SIZE>>
  for StringTableSizeError<'pool, Pe, DIAGNOSTIC_NODE_NAME_SIZE>
{
  fn from(value: UnderlyingProviderReadError<'pool, Pe, DIAGNOSTIC_NODE_NAME_SIZE>) -> Self {
    Self::UnderlyingProviderReadError(value)
  }
}

impl<'pool, Pe: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize>
  Error<DIAGNOSTIC_NODE_NAME_SIZE> for StringTableSizeError<'pool, Pe, DIAGNOSTIC_NODE_NAME_SIZE>
{
  fn with_report<Cb: FnMut(Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, callback: Cb) {
    match self {
      Self::UnderlyingProviderReadError(upre) => upre.with_report(callback),
      Self::InvalidNodeKind(ink) => ink.with_report(callback),
      Self::NotLargeEnough(oob) => oob.with_report(callback),
    }
  }
}
