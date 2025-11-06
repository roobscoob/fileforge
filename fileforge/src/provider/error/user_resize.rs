use crate::error::FileforgeError;

pub trait UserResizeError: FileforgeError {}

impl UserResizeError for core::convert::Infallible {}
