use crate::{error::Error, provider::error::ProviderError};

use super::{
  underlying_provider_read::UnderlyingProviderReadError,
  underlying_provider_stat::UnderlyingProviderStatError,
};

pub enum UnderlyingProviderError<
  'pool,
  Re: ProviderError,
  Se: ProviderError,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
> {
  ReadError(UnderlyingProviderReadError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>),
  StatError(UnderlyingProviderStatError<Se>),
}

impl<'pool, Re: ProviderError, Se: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize>
  Error<DIAGNOSTIC_NODE_NAME_SIZE>
  for UnderlyingProviderError<'pool, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>
{
  fn with_report<Cb: FnMut(crate::error::report::Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(
    &self,
    callback: Cb,
  ) {
    todo!()
  }
}
