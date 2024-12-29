use super::{out_of_bounds::OutOfBoundsError, user_resize::UserResizeError};

pub enum ProviderResizeError<const NODE_NAME_SIZE: usize, UserResize: UserResizeError<NODE_NAME_SIZE>> {
  User(UserResize),
  OutOfBounds(OutOfBoundsError),
}

impl<const NODE_NAME_SIZE: usize, UserResize: UserResizeError<NODE_NAME_SIZE>> From<UserResize> for ProviderResizeError<NODE_NAME_SIZE, UserResize> {
  fn from(user: UserResize) -> Self { Self::User(user) }
}

impl<const NODE_NAME_SIZE: usize, UserResize: UserResizeError<NODE_NAME_SIZE>> From<OutOfBoundsError> for ProviderResizeError<NODE_NAME_SIZE, UserResize> {
  fn from(out_of_bounds: OutOfBoundsError) -> Self { Self::OutOfBounds(out_of_bounds) }
}
