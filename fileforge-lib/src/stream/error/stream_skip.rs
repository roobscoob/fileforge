use super::user_skip::UserSkipError;

pub enum StreamSkipError<const NODE_NAME_SIZE: usize, UserSkip: UserSkipError<NODE_NAME_SIZE>> {
  User(UserSkip),
}
