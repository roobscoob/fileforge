use fileforge_macros::story;
use crate::diagnostic::pool::DiagnosticPoolProvider;
use crate::error::{report::Report, FileforgeError};
use super::{stream_exhausted::StreamExhaustedError, user_mutate::UserMutateError};

#[story("stream failed", StreamMutateError::User(StoryUserError))]
#[story("stream exhausted", StreamMutateError::<StoryUserError>::StreamExhausted(StreamExhaustedError { stream_length: 16, read_length: 4, read_offset: 14 }))]
#[derive(Debug)]
pub enum StreamMutateError<UserMutate: UserMutateError> {
  User(UserMutate),
  StreamExhausted(StreamExhaustedError),
}

impl<UserMutate: UserMutateError> From<UserMutate> for StreamMutateError<UserMutate> {
  fn from(value: UserMutate) -> Self {
    Self::User(value)
  }
}

impl<T, UserRead: UserMutateError, I: From<UserRead>> super::MapExhausted<T, UserRead, I> for Result<T, StreamMutateError<UserRead>> {
  fn map_exhausted<Midpoint: Into<I>>(self, mapper: impl FnOnce(StreamExhaustedError) -> Midpoint) -> Result<T, I> {
    match self {
      Ok(v) => Ok(v),
      Err(StreamMutateError::User(u)) => Err(u.into()),
      Err(StreamMutateError::StreamExhausted(e)) => Err(mapper(e).into()),
    }
  }
}

impl<U: UserMutateError> FileforgeError for StreamMutateError<U> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::User(error) => error.render_into_report(provider, callback),
      Self::StreamExhausted(error) => error.render_into_report(provider, callback),
    }
  }
}
