use crate::error::FileforgeError;

pub trait UserPartitionError<const NODE_NAME_SIZE: usize>: FileforgeError<NODE_NAME_SIZE> {}

impl<const NODE_NAME_SIZE: usize> UserPartitionError<NODE_NAME_SIZE> for core::convert::Infallible {}
