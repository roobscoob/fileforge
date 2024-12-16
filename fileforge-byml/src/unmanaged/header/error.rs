use fileforge_lib::error::Error;
use fileforge_std::endianness::error::EndiannessMarkerError;

pub enum BymlHeaderError<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  Endianness(EndiannessMarkerError<'pool, DIAGNOSTIC_NODE_NAME_SIZE, 2>)
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize> Error<DIAGNOSTIC_NODE_NAME_SIZE> for BymlHeaderError<'pool, DIAGNOSTIC_NODE_NAME_SIZE> {
  fn with_report<Cb: FnMut(fileforge_lib::error::report::Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, callback: Cb) {
    match self {
      BymlHeaderError::Endianness(eme) => eme.with_report(callback)
    }
  }
}