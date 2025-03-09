use crate::error::FileforgeError;

pub trait UserSeekError: FileforgeError {}

impl UserSeekError for core::convert::Infallible {}
