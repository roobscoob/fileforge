use fileforge_lib::{provider::error::ProviderError, error::Error};

use super::get_header::GetHeaderError;

pub enum GetRootNodeError<'pool, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  GetHeaderError(GetHeaderError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>),
}

impl<'pool, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> Error<DIAGNOSTIC_NODE_NAME_SIZE> for GetRootNodeError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE> {
  fn with_report<Cb: FnMut(fileforge_lib::error::report::Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, callback: Cb) {
    match self {
      GetRootNodeError::GetHeaderError(ghe) => ghe.with_report(callback),
    }
  }
}