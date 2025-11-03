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
