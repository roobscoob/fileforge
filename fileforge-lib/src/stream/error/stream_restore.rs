use super::user_restore::UserRestoreError;

#[derive(Debug)]
pub enum StreamRestoreError<UserRestore: UserRestoreError> {
  User(UserRestore),
  CannotRestoreForwards,
}

impl<UserRestore: UserRestoreError> From<UserRestore> for StreamRestoreError<UserRestore> {
  fn from(value: UserRestore) -> Self {
    Self::User(value)
  }
}
