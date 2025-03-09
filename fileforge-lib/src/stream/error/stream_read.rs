use super::{stream_exhausted::StreamExhaustedError, user_read::UserReadError};

pub enum StreamReadError<UserRead: UserReadError> {
  User(UserRead),
  StreamExhausted(StreamExhaustedError),
}