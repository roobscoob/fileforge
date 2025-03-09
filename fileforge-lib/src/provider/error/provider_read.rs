use super::{out_of_bounds::OutOfBoundsError, user_read::UserReadError};

pub enum ProviderReadError<UserRead: UserReadError> {
  User(UserRead),
  OutOfBounds(OutOfBoundsError),
}

impl<UserRead: UserReadError> From<UserRead> for ProviderReadError<UserRead> {
  fn from(user: UserRead) -> Self { Self::User(user) }
}

impl<UserRead: UserReadError> From<OutOfBoundsError> for ProviderReadError< UserRead> {
  fn from(out_of_bounds: OutOfBoundsError) -> Self { Self::OutOfBounds(out_of_bounds) }
}
