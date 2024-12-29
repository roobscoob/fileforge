use super::user_overwrite::UserOverwriteError;

pub enum StreamOverwriteError<const NODE_NAME_SIZE: usize, UserOverwrite: UserOverwriteError<NODE_NAME_SIZE>> {
  User(UserOverwrite),
}
