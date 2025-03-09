use crate::stream::error::user_skip::UserSkipError;

use super::seek_out_of_bounds::SeekOutOfBounds;

pub enum SkipError<'pool, User: UserSkipError> {
    User(User),
    OutOfBounds(SeekOutOfBounds<'pool>),
}