use crate::error::FileforgeError;

pub trait UserMutateError<const NODE_NAME_SIZE: usize>: FileforgeError<NODE_NAME_SIZE> {}

impl<const NODE_NAME_SIZE: usize> UserMutateError<NODE_NAME_SIZE> for core::convert::Infallible {}
