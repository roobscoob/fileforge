use super::{stream_exhausted::StreamExhaustedError, user_read::UserReadError};

pub enum StreamReadError<const NODE_NAME_SIZE: usize, UserRead: for<'pool> UserReadError<'pool, NODE_NAME_SIZE>> {
  User(UserRead),
  StreamExhausted(StreamExhaustedError),
}
