use crate::error::FileforgeError;

pub trait UserReadError<const NODE_NAME_SIZE: usize>: FileforgeError<NODE_NAME_SIZE> {}

impl<const NODE_NAME_SIZE: usize> UserReadError<NODE_NAME_SIZE> for core::convert::Infallible {}
