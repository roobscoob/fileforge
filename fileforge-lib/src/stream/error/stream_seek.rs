use super::{stream_seek_out_of_bounds::StreamSeekOutOfBoundsError, user_seek::UserSeekError};

pub enum StreamSeekError<UserSeek: UserSeekError> {
  User(UserSeek),
  OutOfBounds(StreamSeekOutOfBoundsError),
}

impl<UserSeek: UserSeekError> From<StreamSeekOutOfBoundsError> for StreamSeekError<UserSeek> {
  fn from(value: StreamSeekOutOfBoundsError) -> Self { Self::OutOfBounds(value) }
}

impl<UserSeek: UserSeekError> From<UserSeek> for StreamSeekError<UserSeek> {
  fn from(value: UserSeek) -> Self { Self::User(value) }
}
