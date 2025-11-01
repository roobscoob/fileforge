use fileforge_lib::{binary_reader::error::get_primitive::GetPrimitiveError, error::FileforgeError, stream::error::user_read::UserReadError};

use super::invalid::MagicInvalid;

pub enum MagicError<'pool, const MAGIC_SIZE: usize, U: UserReadError> {
  Failed(GetPrimitiveError<'pool, U>),
  Invalid(MagicInvalid<'pool, MAGIC_SIZE>),
}

impl<'pool, const MAGIC_SIZE: usize, U: UserReadError> From<GetPrimitiveError<'pool, U>> for MagicError<'pool, MAGIC_SIZE, U> {
  fn from(value: GetPrimitiveError<'pool, U>) -> Self {
    MagicError::Failed(value)
  }
}

impl<'pool, const MAGIC_SIZE: usize, U: UserReadError> From<MagicInvalid<'pool, MAGIC_SIZE>> for MagicError<'pool, MAGIC_SIZE, U> {
  fn from(value: MagicInvalid<'pool, MAGIC_SIZE>) -> Self {
    MagicError::Invalid(value)
  }
}

impl<'pool, const MAGIC_SIZE: usize, U: UserReadError> FileforgeError for MagicError<'pool, MAGIC_SIZE, U> {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: fileforge_lib::diagnostic::pool::DiagnosticPoolProvider>(
    &self,
    provider: &'pool_ref P,
    callback: impl for<'tag, 'b, 'p2> FnMut(fileforge_lib::error::report::Report<'tag, 'b, 'p2, 'pool_ref, ITEM_NAME_SIZE, P>) -> (),
  ) {
    unimplemented!()
  }
}
