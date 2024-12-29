use super::user_seek::UserSeekError;

pub enum StreamSeekError<const NODE_NAME_SIZE: usize, UserSeek: UserSeekError<NODE_NAME_SIZE>> {
  User(UserSeek),
}
