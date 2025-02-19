use super::{stream_seek_out_of_bounds::StreamSeekOutOfBoundsError, user_seek::UserSeekError};

pub enum StreamSeekError<const NODE_NAME_SIZE: usize, UserSeek: for<'pool> UserSeekError<'pool, NODE_NAME_SIZE>> {
  User(UserSeek),
  OutOfBounds(StreamSeekOutOfBoundsError),
}

impl<const NODE_NAME_SIZE: usize, UserSeek: for<'pool> UserSeekError<'pool, NODE_NAME_SIZE>> From<StreamSeekOutOfBoundsError> for StreamSeekError<NODE_NAME_SIZE, UserSeek> {
  fn from(value: StreamSeekOutOfBoundsError) -> Self { Self::OutOfBounds(value) }
}

impl<const NODE_NAME_SIZE: usize, UserSeek: for<'pool> UserSeekError<'pool, NODE_NAME_SIZE>> From<UserSeek> for StreamSeekError<NODE_NAME_SIZE, UserSeek> {
  fn from(value: UserSeek) -> Self { Self::User(value) }
}
