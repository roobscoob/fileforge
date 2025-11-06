use crate::error::FileforgeError;

pub trait UserSkipError: FileforgeError {}

impl UserSkipError for core::convert::Infallible {}
