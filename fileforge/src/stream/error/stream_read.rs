use fileforge_macros::story;
use crate::diagnostic::pool::DiagnosticPoolProvider;
use crate::error::{report::Report, FileforgeError};
use super::{stream_exhausted::StreamExhaustedError, user_read::UserReadError};

#[story("stream failed", StreamReadError::User(StoryUserError))]
#[story("stream exhausted", StreamReadError::<StoryUserError>::StreamExhausted(StreamExhaustedError { stream_length: 16, read_length: 4, read_offset: 14 }))]
#[derive(Debug)]
pub enum StreamReadError<UserRead: UserReadError> {
  User(UserRead),
  StreamExhausted(StreamExhaustedError),
}

impl<UserRead: UserReadError> From<UserRead> for StreamReadError<UserRead> {
  fn from(value: UserRead) -> Self {
    Self::User(value)
  }
}

impl<UserRead: UserReadError> From<StreamExhaustedError> for StreamReadError<UserRead> {
  fn from(value: StreamExhaustedError) -> Self {
    Self::StreamExhausted(value)
  }
}

impl<T, UserRead: UserReadError, I: From<UserRead>> super::MapExhausted<T, UserRead, I> for Result<T, StreamReadError<UserRead>> {
  fn map_exhausted<Midpoint: Into<I>>(self, mapper: impl FnOnce(StreamExhaustedError) -> Midpoint) -> Result<T, I> {
    match self {
      Ok(v) => Ok(v),
      Err(StreamReadError::User(u)) => Err(u.into()),
      Err(StreamReadError::StreamExhausted(e)) => Err(mapper(e).into()),
    }
  }
}

impl<U: UserReadError> FileforgeError for StreamReadError<U> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::User(error) => error.render_into_report(provider, callback),
      Self::StreamExhausted(error) => error.render_into_report(provider, callback),
    }
  }
}
