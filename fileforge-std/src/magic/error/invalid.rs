use fileforge::{
  diagnostic::{node::reference::DiagnosticReference, pool::DiagnosticPoolProvider, value::DiagnosticValue},
  error::{report::Report, FileforgeError},
};

use super::super::Magic;

pub struct MagicInvalid<'pool, const MAGIC_SIZE: usize> {
  pub expected: Magic<MAGIC_SIZE>,
  pub actual: DiagnosticValue<'pool, Magic<MAGIC_SIZE>>,
}

impl<'pool, const MAGIC_SIZE: usize> MagicInvalid<'pool, MAGIC_SIZE> {
  pub fn assert(expected: Magic<MAGIC_SIZE>, actual: Magic<MAGIC_SIZE>, get_dr: impl FnOnce() -> Option<DiagnosticReference<'pool>>) -> Result<(), Self> {
    if expected == actual {
      return Ok(());
    }

    Err(MagicInvalid {
      expected,
      actual: DiagnosticValue(actual, get_dr()),
    })
  }
}

impl<'pool, const MAGIC_SIZE: usize> FileforgeError for MagicInvalid<'pool, MAGIC_SIZE> {
  fn render_into_report<P: DiagnosticPoolProvider, const ITEM_NAME_SIZE: usize>(&self, _: P, _: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    unimplemented!()
  }
}
