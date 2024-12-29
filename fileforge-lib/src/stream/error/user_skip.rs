use crate::error::FileforgeError;

pub trait UserSkipError<const NODE_NAME_SIZE: usize>: FileforgeError<NODE_NAME_SIZE> {}

impl<const NODE_NAME_SIZE: usize> UserSkipError<NODE_NAME_SIZE> for core::convert::Infallible {}
