use fileforge_lib::{error::FileforgeError, stream::error::user_read::UserReadError};

use super::decode::Yaz0DecodeError;

pub enum Yaz0DecompressError<'pool, const NODE_NAME_SIZE: usize, StreamReadError: UserReadError<NODE_NAME_SIZE>> {
    Decode(Yaz0DecodeError<'pool, NODE_NAME_SIZE, StreamReadError>)
}

impl<'pool, const NODE_NAME_SIZE: usize, StreamReadError: UserReadError<NODE_NAME_SIZE>> UserReadError<NODE_NAME_SIZE> for Yaz0DecompressError<'pool, NODE_NAME_SIZE, StreamReadError> {}

impl<'pool, const NODE_NAME_SIZE: usize, StreamReadError: UserReadError<NODE_NAME_SIZE>> FileforgeError<'pool, NODE_NAME_SIZE> for Yaz0DecompressError<'pool, NODE_NAME_SIZE, StreamReadError> {
    
}