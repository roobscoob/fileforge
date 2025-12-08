use fileforge::{
  binary_reader::endianness::Endianness,
  diagnostic::{node::reference::DiagnosticReference, pool::DiagnosticPoolProvider, value::DiagnosticValue},
  error::{report::Report, FileforgeError},
};

use super::super::ByteOrderMark;

pub struct ByteOrderMarkInvalid<'pool> {
  pub expected: ByteOrderMark,
  pub actual: DiagnosticValue<'pool, ByteOrderMark>,
}

impl<'pool> ByteOrderMarkInvalid<'pool> {
  pub fn assert(expected: ByteOrderMark, actual: ByteOrderMark, get_dr: impl FnOnce() -> Option<DiagnosticReference<'pool>>) -> Result<Endianness, Self> {
    if expected.le() == actual.le() {
      return Ok(actual.endianness());
    }

    Err(ByteOrderMarkInvalid {
      expected,
      actual: DiagnosticValue(actual, get_dr()),
    })
  }
}

impl<'pool> FileforgeError for ByteOrderMarkInvalid<'pool> {
  fn render_into_report<P: DiagnosticPoolProvider, const ITEM_NAME_SIZE: usize>(&self, _: P, _: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    unimplemented!()
  }
}
