use crate::error::FileforgeError;

use super::{stream_exhausted::StreamExhaustedError, user_overwrite::UserOverwriteError};

#[derive(Debug)]
pub enum StreamOverwriteError<UserOverwrite: UserOverwriteError> {
  User(UserOverwrite),
  StreamExhausted(StreamExhaustedError),
}

impl<UserOverwrite: UserOverwriteError> From<UserOverwrite> for StreamOverwriteError<UserOverwrite> {
  fn from(value: UserOverwrite) -> Self {
    Self::User(value)
  }
}

impl<UserOverwrite: UserOverwriteError> FileforgeError for StreamOverwriteError<UserOverwrite> {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: crate::diagnostic::pool::DiagnosticPoolProvider>(
    &self,
    provider: &'pool_ref P,
    callback: impl for<'tag, 'b, 'pool> FnMut(crate::error::report::Report<'tag, 'b, 'pool, 'pool_ref, ITEM_NAME_SIZE, P>) -> (),
  ) {
    todo!()
  }
}
