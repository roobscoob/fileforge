use crate::stream::error::user_read::UserReadError;

use super::exhausted::ReaderExhaustedError;

pub enum GetPrimitiveError<const NODE_NAME_SIZE: usize, U: UserReadError<NODE_NAME_SIZE>> {
  ReaderExhausted(ReaderExhaustedError),
  User(U),
}
