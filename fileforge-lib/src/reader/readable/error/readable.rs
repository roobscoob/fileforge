use crate::{reader::error::{exhausted::ReaderExhaustedError, get_primitive::GetPrimitiveError}, stream::{error::user_read::UserReadError, ReadableStream}};

use super::user::UserReadableError;

pub enum ReadableError<'pool, const NODE_NAME_SIZE: usize, User: UserReadableError<'pool, NODE_NAME_SIZE>, S: UserReadError<NODE_NAME_SIZE>> {
    GetPrimitiveError(GetPrimitiveError<'pool, NODE_NAME_SIZE, S>),
    StreamExhausted(ReaderExhaustedError<'pool, NODE_NAME_SIZE>),
    StreamUser(S),
    User(User),
}

impl<'pool, const NODE_NAME_SIZE: usize, User: UserReadableError<'pool, NODE_NAME_SIZE>, Stream: UserReadError<NODE_NAME_SIZE>> From<User> for ReadableError<'pool, NODE_NAME_SIZE, User, Stream> {
    fn from(value: User) -> Self {
        Self::User(value)
    }
}

impl<'pool, const NODE_NAME_SIZE: usize, User: UserReadableError<'pool, NODE_NAME_SIZE>, Stream: UserReadError<NODE_NAME_SIZE>>
    From<GetPrimitiveError<'pool, NODE_NAME_SIZE, Stream>>
    for ReadableError<'pool, NODE_NAME_SIZE, User, Stream> {
        fn from(value: GetPrimitiveError<'pool, NODE_NAME_SIZE, Stream>) -> Self {
            Self::GetPrimitiveError(value)
        }
    }

impl<'pool, const NODE_NAME_SIZE: usize, User: UserReadableError<'pool, NODE_NAME_SIZE>, Stream: UserReadError<NODE_NAME_SIZE>> ReadableError<'pool, NODE_NAME_SIZE, User, Stream> {
    pub fn map_user<NewUser: UserReadableError<'pool, NODE_NAME_SIZE>, Mapper: FnOnce(User) -> NewUser>(self, mapper: Mapper) -> ReadableError<'pool, NODE_NAME_SIZE, NewUser, Stream> {
        match self {
            Self::User(old_user) => ReadableError::User(mapper(old_user)),
            Self::StreamUser(stream_user) => ReadableError::StreamUser(stream_user),
            Self::StreamExhausted(stream_exhausted) => ReadableError::StreamExhausted(stream_exhausted),
            Self::GetPrimitiveError(get_primitive_error) => ReadableError::GetPrimitiveError(get_primitive_error),
        }
    }
}

pub trait ReadableErrorResultMapping<'pool, const NODE_NAME_SIZE: usize, User: UserReadableError<'pool, NODE_NAME_SIZE>, S: UserReadError<NODE_NAME_SIZE>, T> {
    fn map_user_err<NewUser: UserReadableError<'pool, NODE_NAME_SIZE>, Mapper: FnOnce(User) -> NewUser>(self, mapper: Mapper) -> Result<T, ReadableError<'pool, NODE_NAME_SIZE, NewUser, S>>;
}

impl<'pool, const NODE_NAME_SIZE: usize, User: UserReadableError<'pool, NODE_NAME_SIZE>, S: UserReadError<NODE_NAME_SIZE>, T>
    ReadableErrorResultMapping<'pool, NODE_NAME_SIZE, User, S, T>
    for Result<T, ReadableError<'pool, NODE_NAME_SIZE, User, S>> {
        fn map_user_err<NewUser: UserReadableError<'pool, NODE_NAME_SIZE>, Mapper: FnOnce(User) -> NewUser>(self, mapper: Mapper) -> Result<T, ReadableError<'pool, NODE_NAME_SIZE, NewUser, S>> {
            self.map_err(|e| e.map_user(mapper))
        }
    }