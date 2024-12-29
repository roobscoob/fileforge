use crate::error::FileforgeError;

pub trait UserWriteError<const NODE_NAME_SIZE: usize>: FileforgeError<NODE_NAME_SIZE> {}

impl<const NODE_NAME_SIZE: usize> UserWriteError<NODE_NAME_SIZE> for core::convert::Infallible {}
