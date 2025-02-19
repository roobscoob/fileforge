use super::{stream_seek_out_of_bounds::StreamSeekOutOfBoundsError, user_skip::UserSkipError};

pub enum StreamSkipError<const NODE_NAME_SIZE: usize, UserSkip: for<'pool> UserSkipError<'pool, NODE_NAME_SIZE>> {
  User(UserSkip),
  OutOfBounds(StreamSeekOutOfBoundsError),

  // ASSERT: offset + seek_forwards_distance > u64::MAX
  SeekPointOverflowed { stream_length: u64, offset: u64, seek_forwards_distance: u64 },
}

impl<const NODE_NAME_SIZE: usize, UserSkip: for<'pool> UserSkipError<'pool, NODE_NAME_SIZE>> From<StreamSeekOutOfBoundsError> for StreamSkipError<NODE_NAME_SIZE, UserSkip> {
  fn from(value: StreamSeekOutOfBoundsError) -> Self { Self::OutOfBounds(value) }
}

impl<const NODE_NAME_SIZE: usize, UserSkip: for<'pool> UserSkipError<'pool, NODE_NAME_SIZE>> From<UserSkip> for StreamSkipError<NODE_NAME_SIZE, UserSkip> {
  fn from(value: UserSkip) -> Self { Self::User(value) }
}

impl<const NODE_NAME_SIZE: usize, UserSkip: for<'pool> UserSkipError<'pool, NODE_NAME_SIZE>> StreamSkipError<NODE_NAME_SIZE, UserSkip> {
  pub fn assert_relative_forwards(stream_length: u64, offset: u64, relative_forwards: u64) -> Result<u64, Self> {
    let seek_point = offset.checked_add(relative_forwards).ok_or(Self::SeekPointOverflowed {
      stream_length,
      offset,
      seek_forwards_distance: relative_forwards,
    })?;

    StreamSeekOutOfBoundsError::assert(stream_length, seek_point)?;

    Ok(seek_point)
  }
}
