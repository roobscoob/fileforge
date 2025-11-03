use super::{out_of_bounds::OutOfBoundsError, user_slice::UserSliceError};

#[derive(Debug)]
pub enum ProviderSliceError<UserSlice: UserSliceError> {
  User(UserSlice),
  OutOfBounds(OutOfBoundsError),
}

impl<UserSlice: UserSliceError> From<UserSlice> for ProviderSliceError<UserSlice> {
  fn from(user: UserSlice) -> Self {
    Self::User(user)
  }
}

impl<UserSlice: UserSliceError> From<OutOfBoundsError> for ProviderSliceError<UserSlice> {
  fn from(out_of_bounds: OutOfBoundsError) -> Self {
    Self::OutOfBounds(out_of_bounds)
  }
}
