use fileforge_lib::{
  error::{report::Report, Error},
  provider::{error::ProviderError, r#trait::Provider},
  reader::error::{
    underlying_provider_error::UnderlyingProviderError,
    underlying_provider_read::UnderlyingProviderReadError,
  },
};

pub enum GetLengthError<'pool, P: Provider, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  UnderlyingProviderError(
    UnderlyingProviderError<'pool, P::ReadError, P::StatError, DIAGNOSTIC_NODE_NAME_SIZE>,
  ),
  NotLargeEnough(StringTableNotLargeEnough<P::StatError>),
}

impl<'pool, P: Provider, const DIAGNOSTIC_NODE_NAME_SIZE: usize> Error<DIAGNOSTIC_NODE_NAME_SIZE>
  for GetLengthError<'pool, P, DIAGNOSTIC_NODE_NAME_SIZE>
{
  fn with_report<Cb: FnMut(Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, callback: Cb) {
    todo!()
  }
}

pub struct StringTableNotLargeEnough<Se: ProviderError> {
  pub desired_length: Option<u64>,
  pub available_length: Result<u64, Se>,
}

impl<Se: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> Error<DIAGNOSTIC_NODE_NAME_SIZE>
  for StringTableNotLargeEnough<Se>
{
  fn with_report<Cb: FnMut(Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, callback: Cb) {
    todo!()
  }
}
