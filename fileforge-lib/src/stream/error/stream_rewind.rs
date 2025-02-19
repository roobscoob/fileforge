use super::user_rewind::UserRewindError;

pub enum StreamRewindError<const NODE_NAME_SIZE: usize, UserRewind: for<'pool> UserRewindError<'pool, NODE_NAME_SIZE>> {
  User(UserRewind),

  // ASSERT: offset - seek_backwards_distance < u64::MIN
  SeekPointUnderflowed { stream_length: u64, offset: u64, seek_backwards_distance: u64 },
}

impl<const NODE_NAME_SIZE: usize, UserRewind: for<'pool> UserRewindError<'pool, NODE_NAME_SIZE>> StreamRewindError<NODE_NAME_SIZE, UserRewind> {
  pub fn assert_relative_backwards(stream_length: u64, offset: u64, relative_backwards: u64) -> Result<u64, Self> {
    let seek_point = offset.checked_sub(relative_backwards).ok_or(Self::SeekPointUnderflowed {
      stream_length,
      offset,
      seek_backwards_distance: relative_backwards,
    })?;

    Ok(seek_point)
  }
}
