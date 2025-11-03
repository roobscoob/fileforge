use super::{out_of_bounds::OutOfBoundsError, user_mutate::UserMutateError};

#[derive(Debug)]
pub enum ProviderMutateError<UserMutate: UserMutateError> {
  User(UserMutate),
  OutOfBounds(OutOfBoundsError),
}

impl<UserMutate: UserMutateError> From<UserMutate> for ProviderMutateError<UserMutate> {
  fn from(user: UserMutate) -> Self {
    Self::User(user)
  }
}

impl<UserMutate: UserMutateError> From<OutOfBoundsError> for ProviderMutateError<UserMutate> {
  fn from(out_of_bounds: OutOfBoundsError) -> Self {
    Self::OutOfBounds(out_of_bounds)
  }
}
