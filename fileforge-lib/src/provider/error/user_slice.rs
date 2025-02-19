use crate::error::FileforgeError;

pub trait UserSliceError<const NODE_NAME_SIZE: usize>: for<'pool> FileforgeError<'pool, NODE_NAME_SIZE> {}

impl<const NODE_NAME_SIZE: usize> UserSliceError<NODE_NAME_SIZE> for core::convert::Infallible {}
