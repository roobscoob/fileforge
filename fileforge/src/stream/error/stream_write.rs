use super::user_write::UserWriteError;

pub enum StreamWriteError<UserWrite: UserWriteError> {
  User(UserWrite),
}
