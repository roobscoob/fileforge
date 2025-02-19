use super::user_write::UserWriteError;

pub enum StreamWriteError<'pool, const NODE_NAME_SIZE: usize, UserWrite: UserWriteError<'pool, NODE_NAME_SIZE>> {
  User(UserWrite),
}
