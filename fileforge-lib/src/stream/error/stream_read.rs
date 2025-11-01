use super::{stream_exhausted::StreamExhaustedError, user_read::UserReadError};

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
