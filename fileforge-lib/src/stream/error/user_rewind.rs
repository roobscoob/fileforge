use crate::error::FileforgeError;

pub trait UserRewindError<const NODE_NAME_SIZE: usize>: FileforgeError<NODE_NAME_SIZE> {}

impl<const NODE_NAME_SIZE: usize> UserRewindError<NODE_NAME_SIZE> for core::convert::Infallible {}
