use crate::error::FileforgeError;

pub trait UserSkipError<'pool, const NODE_NAME_SIZE: usize>: FileforgeError<'pool, NODE_NAME_SIZE> {}

impl<'pool, const NODE_NAME_SIZE: usize> UserSkipError<'pool, NODE_NAME_SIZE> for core::convert::Infallible {}
