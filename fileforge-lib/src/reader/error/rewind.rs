use crate::stream::error::user_rewind::UserRewindError;

use super::seek_out_of_bounds::SeekOutOfBounds;

pub enum RewindError<'pool, User: UserRewindError> {
    User(User),
    OutOfBounds(SeekOutOfBounds<'pool>),
}