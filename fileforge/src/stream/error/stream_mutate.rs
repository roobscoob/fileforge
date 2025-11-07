use super::{stream_exhausted::StreamExhaustedError, user_mutate::UserMutateError};

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
