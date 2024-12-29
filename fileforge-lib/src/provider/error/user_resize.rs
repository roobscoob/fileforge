use crate::error::FileforgeError;

pub trait UserResizeError<const NODE_NAME_SIZE: usize>: FileforgeError<NODE_NAME_SIZE> {}

impl<const NODE_NAME_SIZE: usize> UserResizeError<NODE_NAME_SIZE> for core::convert::Infallible {}
