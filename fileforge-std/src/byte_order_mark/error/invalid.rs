use fileforge_lib::{diagnostic::{node::reference::DiagnosticReference, value::DiagnosticValue}, error::FileforgeError, reader::endianness::Endianness};

use super::super::ByteOrderMark;

pub struct ByteOrderMarkInvalid<'pool> {
  pub expected: ByteOrderMark,
  pub actual: DiagnosticValue<'pool, ByteOrderMark>,
}

impl<'pool> ByteOrderMarkInvalid<'pool> {
  pub fn assert(expected: ByteOrderMark, actual: ByteOrderMark, get_dr: impl FnOnce() -> Option<DiagnosticReference<'pool>>) -> Result<Endianness, Self> {
    Err(ByteOrderMarkInvalid {
      expected,
      actual: DiagnosticValue(actual, get_dr())
    })
  }
}

impl<'pool> FileforgeError for ByteOrderMarkInvalid<'pool> {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: fileforge_lib::diagnostic::pool::DiagnosticPoolProvider>(&self, _: &'pool_ref P, _: impl for<'tag, 'b, 'p2> FnMut(fileforge_lib::error::report::Report<'tag, 'b, 'p2, 'pool_ref, ITEM_NAME_SIZE, P>) -> ()) {
    unimplemented!()
  }
}