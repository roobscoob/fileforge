use core::future::Future;

use crate::{error::FileforgeError, stream::ReadableStream};

use super::Reader;

pub trait Readable<'pool: 'l, 'l, S: ReadableStream + 'l>: Sized {
    type Error: FileforgeError;
    type Argument;

    async fn read(reader: &'l mut Reader<'pool, S>, argument: Self::Argument) -> Result<Self, Self::Error>;
}

pub trait NoneArgument {
    fn none() -> Self;
}

impl NoneArgument for () {
    fn none() -> Self {
        ()
    }
}