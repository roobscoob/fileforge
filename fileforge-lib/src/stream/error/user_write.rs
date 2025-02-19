use crate::error::FileforgeError;

pub trait UserWriteError<'pool, const NODE_NAME_SIZE: usize>: FileforgeError<'pool, NODE_NAME_SIZE> {}

impl<'pool, const NODE_NAME_SIZE: usize> UserWriteError<'pool, NODE_NAME_SIZE> for core::convert::Infallible {}
