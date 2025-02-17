use super::{stream_exhausted::StreamExhaustedError, user_mutate::UserMutateError};

pub enum StreamMutateError<const NODE_NAME_SIZE: usize, UserMutate: UserMutateError<NODE_NAME_SIZE>> {
  User(UserMutate),
  StreamExhausted(StreamExhaustedError),
}
