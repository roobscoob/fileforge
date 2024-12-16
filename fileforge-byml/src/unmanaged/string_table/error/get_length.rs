use fileforge_lib::{error::Error, provider::r#trait::Provider, reader::error::underlying_provider_read::UnderlyingProviderReadError};

pub enum GetLengthError<'pool, P: Provider, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  UnderlyingProviderError(UnderlyingProviderReadError<'pool, P::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>),
  NotLargeEnough(StringTableNotLargeEnough),
}

impl<'pool, P: Provider, const DIAGNOSTIC_NODE_NAME_SIZE: usize> Error<DIAGNOSTIC_NODE_NAME_SIZE> for GetLengthError<'pool, P, DIAGNOSTIC_NODE_NAME_SIZE> {
  fn with_report<Cb: FnMut(fileforge_lib::error::report::Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, callback: Cb) {
    todo!()
  }
}

pub struct StringTableNotLargeEnough {
  pub desired_length: u64,
  pub available_length: u64,
}

impl<const DIAGNOSTIC_NODE_NAME_SIZE: usize> Error<DIAGNOSTIC_NODE_NAME_SIZE> for  StringTableNotLargeEnough {
  fn with_report<Cb: FnMut(fileforge_lib::error::report::Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, callback: Cb) {
    todo!()
  }
}