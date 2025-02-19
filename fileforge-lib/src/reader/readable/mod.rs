use core::future::Future;

use error::{readable::ReadableError, user::UserReadableError};

use crate::stream::ReadableStream;

use super::Reader;

pub mod error;

pub trait Readable<'pool, 'l, const NODE_NAME_SIZE: usize>: Sized {
    type Error<S: ReadableStream<NODE_NAME_SIZE> + 'l>: UserReadableError<'pool, NODE_NAME_SIZE> where 'pool: 'l;
    type Argument;

    fn read<S: ReadableStream<NODE_NAME_SIZE>>(reader: &'l mut Reader<'pool, NODE_NAME_SIZE, S>, argument: Self::Argument) -> impl Future<Output = Result<Self, ReadableError<'pool, NODE_NAME_SIZE, Self::Error<S>, S::ReadError>>>;
}

pub trait NoneArgument {
    fn none() -> Self;
}

impl NoneArgument for () {
    fn none() -> Self {
        ()
    }
}