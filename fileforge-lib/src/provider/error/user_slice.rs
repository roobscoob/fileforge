use crate::error::FileforgeError;

pub trait UserSliceError: FileforgeError {}

impl UserSliceError for core::convert::Infallible {}
