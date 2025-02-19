use super::{stream_exhausted::StreamExhaustedError, user_mutate::UserMutateError};

pub enum StreamMutateError<const NODE_NAME_SIZE: usize, UserMutate: for<'pool> UserMutateError<'pool, NODE_NAME_SIZE>> {
  User(UserMutate),
  StreamExhausted(StreamExhaustedError),
}
