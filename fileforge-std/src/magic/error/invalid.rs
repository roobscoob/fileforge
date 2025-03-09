use fileforge_lib::{diagnostic::{node::reference::DiagnosticReference, value::DiagnosticValue}, error::FileforgeError};

use super::super::Magic;

pub struct MagicInvalid<'pool, const MAGIC_SIZE: usize> {
  pub expected: Magic<MAGIC_SIZE>,
  pub actual: DiagnosticValue<'pool, Magic<MAGIC_SIZE>>,
}

impl<'pool, const MAGIC_SIZE: usize> MagicInvalid<'pool, MAGIC_SIZE> {
  pub fn assert(expected: Magic<MAGIC_SIZE>, actual: Magic<MAGIC_SIZE>, get_dr: impl FnOnce() -> Option<DiagnosticReference<'pool>>) -> Result<(), Self> {
    if expected == actual {
      return Ok(())
    }

    Err(MagicInvalid {
      expected,
      actual: DiagnosticValue(actual, get_dr())
    })
  }
}

impl<'pool, const MAGIC_SIZE: usize> FileforgeError for MagicInvalid<'pool, MAGIC_SIZE> {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: fileforge_lib::diagnostic::pool::DiagnosticPoolProvider>(&self, _: &'pool_ref P, _: impl for<'tag, 'b, 'p2> FnMut(fileforge_lib::error::report::Report<'tag, 'b, 'p2, 'pool_ref, ITEM_NAME_SIZE, P>) -> ()) {
    unimplemented!()
  }
}