use super::user_write::UserWriteError;

pub enum StreamWriteError<const NODE_NAME_SIZE: usize, UserWrite: UserWriteError<NODE_NAME_SIZE>> {
  User(UserWrite),
}
