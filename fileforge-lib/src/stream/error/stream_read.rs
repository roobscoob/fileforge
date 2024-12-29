use super::{stream_exhausted::StreamExhaustedError, user_read::UserReadError};

pub enum StreamReadError<const NODE_NAME_SIZE: usize, UserRead: UserReadError<NODE_NAME_SIZE>> {
  User(UserRead),
  StreamExhausted(StreamExhaustedError),
}
