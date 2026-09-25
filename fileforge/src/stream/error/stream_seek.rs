use fileforge_macros::story;
use crate::diagnostic::pool::DiagnosticPoolProvider;
use crate::error::{report::Report, FileforgeError};
use super::{stream_seek_out_of_bounds::StreamSeekOutOfBoundsError, user_seek::UserSeekError};

#[story("stream failed", StreamSeekError::User(StoryUserError))]
#[story("out of bounds", StreamSeekError::<StoryUserError>::OutOfBounds(StreamSeekOutOfBoundsError { stream_length: 16, seek_point: 40 }))]
pub enum StreamSeekError<UserSeek: UserSeekError> {
  User(UserSeek),
  OutOfBounds(StreamSeekOutOfBoundsError),
}

impl<UserSeek: UserSeekError> From<StreamSeekOutOfBoundsError> for StreamSeekError<UserSeek> {
  fn from(value: StreamSeekOutOfBoundsError) -> Self { Self::OutOfBounds(value) }
}

impl<UserSeek: UserSeekError> From<UserSeek> for StreamSeekError<UserSeek> {
  fn from(value: UserSeek) -> Self { Self::User(value) }
}

impl<U: UserSeekError> FileforgeError for StreamSeekError<U> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::User(error) => error.render_into_report(provider, callback),
      Self::OutOfBounds(error) => error.render_into_report(provider, callback),
    }
  }
}
