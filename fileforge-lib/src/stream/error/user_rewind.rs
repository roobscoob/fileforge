use crate::error::FileforgeError;

pub trait UserRewindError: FileforgeError {}

impl UserRewindError for core::convert::Infallible {}
