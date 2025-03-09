use crate::error::FileforgeError;

pub trait UserWriteError: FileforgeError {}

impl UserWriteError for core::convert::Infallible {}
