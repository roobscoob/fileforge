use super::{stream_exhausted::StreamExhaustedError, user_overwrite::UserOverwriteError};

pub enum StreamOverwriteError<const NODE_NAME_SIZE: usize, UserOverwrite: for<'pool> UserOverwriteError<'pool, NODE_NAME_SIZE>> {
  User(UserOverwrite),
  StreamExhausted(StreamExhaustedError),
}
