use crate::error::FileforgeError;

pub trait UserPartitionError: FileforgeError {}

impl UserPartitionError for core::convert::Infallible {}
