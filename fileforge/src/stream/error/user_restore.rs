use crate::error::FileforgeError;

pub trait UserRestoreError: FileforgeError {}

impl UserRestoreError for core::convert::Infallible {}
