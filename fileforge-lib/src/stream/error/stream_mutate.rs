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
