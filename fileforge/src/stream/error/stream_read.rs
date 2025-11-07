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

impl<T, UserRead: UserReadError, I: From<UserRead>> super::MapExhausted<T, UserRead, I> for Result<T, StreamReadError<UserRead>> {
  fn map_exhausted<Midpoint: Into<I>>(self, mapper: impl FnOnce(StreamExhaustedError) -> Midpoint) -> Result<T, I> {
    match self {
      Ok(v) => Ok(v),
      Err(StreamReadError::User(u)) => Err(u.into()),
      Err(StreamReadError::StreamExhausted(e)) => Err(mapper(e).into()),
    }
  }
}
