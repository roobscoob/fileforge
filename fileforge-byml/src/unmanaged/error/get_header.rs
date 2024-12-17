use fileforge_lib::{
  error::{report::Report, Error},
  provider::error::ProviderError,
  reader::error::{
    out_of_bounds::ReadOutOfBoundsError, underlying_provider_read::UnderlyingProviderReadError,
  },
};

use crate::unmanaged::header::error::BymlHeaderError;

pub enum GetHeaderError<'pool, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  HeaderError(BymlHeaderError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>),
  MissingData(ReadOutOfBoundsError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>),
  ProviderError(UnderlyingProviderReadError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>),
}

impl<'pool, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize>
  Error<DIAGNOSTIC_NODE_NAME_SIZE> for GetHeaderError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>
{
  fn with_report<Cb: FnMut(Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, callback: Cb) {
    match self {
      GetHeaderError::ProviderError(pe) => pe.with_report(callback),
      GetHeaderError::MissingData(oob) => oob.with_report(callback),
      GetHeaderError::HeaderError(bhe) => bhe.with_report(callback),
    }
  }
}
