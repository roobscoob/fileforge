use crate::{error::FileforgeError, provider::error::user_read::UserReadError};

use super::read_out_of_bounds::ReadOutOfBounds;

pub enum ReadBytesError<'pool, const NODE_NAME_SIZE: usize, Re: UserReadError<NODE_NAME_SIZE>> {
  ReadOutOfBounds(ReadOutOfBounds<'pool, NODE_NAME_SIZE>),
  User(Re),
}

impl<'pool, const NODE_NAME_SIZE: usize, Re: UserReadError<NODE_NAME_SIZE>> FileforgeError<NODE_NAME_SIZE> for ReadBytesError<'pool, NODE_NAME_SIZE, Re> {
  fn render_into_report(&self, callback: impl FnMut(crate::error::report::Report<NODE_NAME_SIZE>) -> ()) {
    match self {
      ReadBytesError::ReadOutOfBounds(roob) => roob.render_into_report(callback),
      ReadBytesError::User(u) => u.render_into_report(callback),
    }
  }
}
