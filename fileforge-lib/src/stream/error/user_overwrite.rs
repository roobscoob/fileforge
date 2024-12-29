use crate::error::FileforgeError;

pub trait UserOverwriteError<const NODE_NAME_SIZE: usize>: FileforgeError<NODE_NAME_SIZE> {}

impl<const NODE_NAME_SIZE: usize> UserOverwriteError<NODE_NAME_SIZE> for core::convert::Infallible {}
