use fileforge_macros::story;
use crate::diagnostic::pool::DiagnosticPoolProvider;
use crate::error::{report::Report, FileforgeError};
use super::user_write::UserWriteError;

#[story("stream failed", StreamWriteError::User(StoryUserError))]
pub enum StreamWriteError<UserWrite: UserWriteError> {
  User(UserWrite),
}

impl<U: UserWriteError> FileforgeError for StreamWriteError<U> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::User(error) => error.render_into_report(provider, callback),
    }
  }
}
