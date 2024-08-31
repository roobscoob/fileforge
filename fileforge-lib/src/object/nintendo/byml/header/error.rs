use crate::{error::Error, provider::error::ProviderError, reader::error::ParseError, object::endianness::error::EndiannessMarkerError};

pub enum BymlHeaderError<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  Endianness(EndiannessMarkerError<'pool, DIAGNOSTIC_NODE_NAME_SIZE, 2>)
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize> Error<DIAGNOSTIC_NODE_NAME_SIZE> for BymlHeaderError<'pool, DIAGNOSTIC_NODE_NAME_SIZE> {
  fn with_report<Cb: FnMut(crate::error::report::Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, callback: Cb) {
    
  }
}