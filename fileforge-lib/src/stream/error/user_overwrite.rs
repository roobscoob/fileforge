use crate::error::FileforgeError;

pub trait UserOverwriteError: FileforgeError {}

impl UserOverwriteError for core::convert::Infallible {}
