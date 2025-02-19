use fileforge_lib::{diagnostic::{node::reference::DiagnosticReference, value::DiagnosticValue}, error::FileforgeError, reader::readable::error::user::UserReadableError};

use super::Magic;

pub struct MagicError<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, const MAGIC_SIZE: usize> {
  pub expected: Magic<MAGIC_SIZE>,
  pub actual: DiagnosticValue<'pool, Magic<MAGIC_SIZE>, DIAGNOSTIC_NODE_NAME_SIZE>,
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, const MAGIC_SIZE: usize> MagicError<'pool, DIAGNOSTIC_NODE_NAME_SIZE, MAGIC_SIZE> {
  pub fn assert(expected: Magic<MAGIC_SIZE>, actual: Magic<MAGIC_SIZE>, get_dr: impl FnOnce() -> Option<DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>>) -> Result<(), Self> {
    if expected == actual {
      return Ok(())
    }

    Err(MagicError {
      expected,
      actual: DiagnosticValue(actual, get_dr())
    })
  }
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, const MAGIC_SIZE: usize> UserReadableError<'pool, DIAGNOSTIC_NODE_NAME_SIZE> for MagicError<'pool, DIAGNOSTIC_NODE_NAME_SIZE, MAGIC_SIZE> {}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, const MAGIC_SIZE: usize> FileforgeError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>  for MagicError<'pool, DIAGNOSTIC_NODE_NAME_SIZE, MAGIC_SIZE> {
    fn render_into_report(&self, callback: impl for<'a, 'b> FnMut(fileforge_lib::error::report::Report<'a, 'b, 'pool, DIAGNOSTIC_NODE_NAME_SIZE>) -> ()) {
        unimplemented!()
    }
}