use fileforge_lib::{
  error::{report::Report, Error},
  provider::error::ProviderError,
  reader::error::underlying_provider_stat::UnderlyingProviderStatError,
};

use super::{
  get_header::GetHeaderError, header_root_node_data_out_of_bounds::HeaderRootNodeOutOfBounds,
};

pub enum GetRootNodeError<
  'pool,
  Re: ProviderError,
  Se: ProviderError,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
> {
  FailedToStat(UnderlyingProviderStatError<Se>),
  GetHeaderError(GetHeaderError<'pool, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>),
  HederRootNodeOutOfBounds(HeaderRootNodeOutOfBounds),
  ReadErrorWhileReadingType(),
}

impl<'pool, Re: ProviderError, Se: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize>
  Error<DIAGNOSTIC_NODE_NAME_SIZE> for GetRootNodeError<'pool, Re, Se, DIAGNOSTIC_NODE_NAME_SIZE>
{
  fn with_report<Cb: FnMut(Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, callback: Cb) {
    match self {
      GetRootNodeError::FailedToStat(upse) => upse.with_report(callback),
      GetRootNodeError::GetHeaderError(ghe) => ghe.with_report(callback),
      GetRootNodeError::HederRootNodeOutOfBounds(oob) => todo!(),
      GetRootNodeError::ReadErrorWhileReadingType() => todo!(),
    }
  }
}
