use super::{stream_exhausted::StreamExhaustedError, user_partition::UserPartitionError};

pub enum StreamPartitionError<UserPartition: UserPartitionError> {
  User(UserPartition),
  StreamExhausted(StreamExhaustedError),
}
