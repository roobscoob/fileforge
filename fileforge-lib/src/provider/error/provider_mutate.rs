use super::{out_of_bounds::OutOfBoundsError, user_mutate::UserMutateError};

pub enum ProviderMutateError<const NODE_NAME_SIZE: usize, UserMutate: UserMutateError<NODE_NAME_SIZE>> {
  User(UserMutate),
  OutOfBounds(OutOfBoundsError),
}

impl<const NODE_NAME_SIZE: usize, UserMutate: UserMutateError<NODE_NAME_SIZE>> From<UserMutate> for ProviderMutateError<NODE_NAME_SIZE, UserMutate> {
  fn from(user: UserMutate) -> Self { Self::User(user) }
}

impl<const NODE_NAME_SIZE: usize, UserMutate: UserMutateError<NODE_NAME_SIZE>> From<OutOfBoundsError> for ProviderMutateError<NODE_NAME_SIZE, UserMutate> {
  fn from(out_of_bounds: OutOfBoundsError) -> Self { Self::OutOfBounds(out_of_bounds) }
}
