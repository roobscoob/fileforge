use crate::error::FileforgeError;

pub trait UserReadError: FileforgeError {}

impl UserReadError for core::convert::Infallible {}
