use super::{out_of_bounds::OutOfBoundsError, user_read::UserReadError};

pub enum ProviderReadError<const NODE_NAME_SIZE: usize, UserRead: UserReadError<NODE_NAME_SIZE>> {
  User(UserRead),
  OutOfBounds(OutOfBoundsError),
}

impl<const NODE_NAME_SIZE: usize, UserRead: UserReadError<NODE_NAME_SIZE>> From<UserRead> for ProviderReadError<NODE_NAME_SIZE, UserRead> {
  fn from(user: UserRead) -> Self { Self::User(user) }
}

impl<const NODE_NAME_SIZE: usize, UserRead: UserReadError<NODE_NAME_SIZE>> From<OutOfBoundsError> for ProviderReadError<NODE_NAME_SIZE, UserRead> {
  fn from(out_of_bounds: OutOfBoundsError) -> Self { Self::OutOfBounds(out_of_bounds) }
}
