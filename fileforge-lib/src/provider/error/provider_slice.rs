use super::{out_of_bounds::OutOfBoundsError, user_slice::UserSliceError};

pub enum ProviderSliceError<const NODE_NAME_SIZE: usize, UserSlice: UserSliceError<NODE_NAME_SIZE>> {
  User(UserSlice),
  OutOfBounds(OutOfBoundsError),
}

impl<const NODE_NAME_SIZE: usize, UserSlice: UserSliceError<NODE_NAME_SIZE>> From<UserSlice> for ProviderSliceError<NODE_NAME_SIZE, UserSlice> {
  fn from(user: UserSlice) -> Self { Self::User(user) }
}

impl<const NODE_NAME_SIZE: usize, UserSlice: UserSliceError<NODE_NAME_SIZE>> From<OutOfBoundsError> for ProviderSliceError<NODE_NAME_SIZE, UserSlice> {
  fn from(out_of_bounds: OutOfBoundsError) -> Self { Self::OutOfBounds(out_of_bounds) }
}
