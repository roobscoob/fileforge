use fileforge_macros::story;
use crate::diagnostic::pool::DiagnosticPoolProvider;
use crate::error::{report::Report, FileforgeError};
use super::{stream_exhausted::StreamExhaustedError, user_overwrite::UserOverwriteError};

#[story("stream failed", StreamOverwriteError::User(StoryUserError))]
#[story("stream exhausted", StreamOverwriteError::<StoryUserError>::StreamExhausted(StreamExhaustedError { stream_length: 16, read_length: 4, read_offset: 14 }))]
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

impl<U: UserOverwriteError> FileforgeError for StreamOverwriteError<U> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::User(error) => error.render_into_report(provider, callback),
      Self::StreamExhausted(error) => error.render_into_report(provider, callback),
    }
  }
}
