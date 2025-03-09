use super::{stream_exhausted::StreamExhaustedError, user_overwrite::UserOverwriteError};

pub enum StreamOverwriteError<UserOverwrite: UserOverwriteError> {
  User(UserOverwrite),
  StreamExhausted(StreamExhaustedError),
}
