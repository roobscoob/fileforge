use super::{out_of_bounds::OutOfBoundsError, user_resize::UserResizeError};

#[derive(Debug)]
pub enum ProviderResizeError<UserResize: UserResizeError> {
  User(UserResize),
  OutOfBounds(OutOfBoundsError),
}

impl<UserResize: UserResizeError> From<UserResize> for ProviderResizeError<UserResize> {
  fn from(user: UserResize) -> Self {
    Self::User(user)
  }
}

impl<UserResize: UserResizeError> From<OutOfBoundsError> for ProviderResizeError<UserResize> {
  fn from(out_of_bounds: OutOfBoundsError) -> Self {
    Self::OutOfBounds(out_of_bounds)
  }
}
