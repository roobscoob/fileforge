use crate::error::FileforgeError;

pub trait UserMutateError: FileforgeError {}

impl UserMutateError for core::convert::Infallible {}
