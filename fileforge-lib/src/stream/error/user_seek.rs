use crate::error::FileforgeError;

pub trait UserSeekError<const NODE_NAME_SIZE: usize>: FileforgeError<NODE_NAME_SIZE> {}

impl<const NODE_NAME_SIZE: usize> UserSeekError<NODE_NAME_SIZE> for core::convert::Infallible {}
