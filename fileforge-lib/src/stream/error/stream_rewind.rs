use super::user_rewind::UserRewindError;

pub enum StreamRewindError<const NODE_NAME_SIZE: usize, UserRewind: UserRewindError<NODE_NAME_SIZE>> {
  User(UserRewind),
}
