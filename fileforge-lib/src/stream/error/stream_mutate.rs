use super::{stream_exhausted::StreamExhaustedError, user_mutate::UserMutateError};

pub enum StreamMutateError<UserMutate: UserMutateError> {
  User(UserMutate),
  StreamExhausted(StreamExhaustedError),
}
