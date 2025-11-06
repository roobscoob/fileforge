use crate::provider::error::user_partition::UserPartitionError;

use super::out_of_bounds::OutOfBoundsError;

#[derive(Debug)]
pub enum ProviderPartitionError<UserPartition: UserPartitionError> {
  User(UserPartition),
  OutOfBounds(OutOfBoundsError),
}

impl<UserPartition: UserPartitionError> From<UserPartition> for ProviderPartitionError<UserPartition> {
  fn from(user: UserPartition) -> Self {
    Self::User(user)
  }
}

impl<UserPartition: UserPartitionError> From<OutOfBoundsError> for ProviderPartitionError<UserPartition> {
  fn from(out_of_bounds: OutOfBoundsError) -> Self {
    Self::OutOfBounds(out_of_bounds)
  }
}
