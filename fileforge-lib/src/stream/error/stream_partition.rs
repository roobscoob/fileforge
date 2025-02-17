use super::{stream_exhausted::StreamExhaustedError, user_partition::UserPartitionError};

pub enum StreamPartitionError<const NODE_NAME_SIZE: usize, UserPartition: UserPartitionError<NODE_NAME_SIZE>> {
  User(UserPartition),
  StreamExhausted(StreamExhaustedError),
}
