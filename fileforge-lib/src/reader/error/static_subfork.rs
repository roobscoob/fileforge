use crate::{error::FileforgeError, stream::StaticPartitionableStream};

use super::seek_out_of_bounds::SeekOutOfBounds;

pub enum StaticSubforkError<'l, 'pool, const SIZE: usize, const NODE_NAME_SIZE: usize, S: StaticPartitionableStream<'l, NODE_NAME_SIZE, SIZE>> {
  Stream(S::PartitionError),
  OutOfBounds(SeekOutOfBounds<'pool, NODE_NAME_SIZE>),
}

impl<'l, 'pool, const SIZE: usize, const NODE_NAME_SIZE: usize, S: StaticPartitionableStream<'l, NODE_NAME_SIZE, SIZE>> FileforgeError<'pool, NODE_NAME_SIZE> for StaticSubforkError<'l, 'pool, SIZE, NODE_NAME_SIZE, S> {
  fn render_into_report(&self, callback: impl for<'a, 'b> FnMut(crate::error::report::Report<'a, 'b, 'pool, NODE_NAME_SIZE>) -> ()) {
    match self {
      Self::Stream(s) => s.render_into_report(callback),
      Self::OutOfBounds(oob) => oob.render_into_report(callback),
    }
  }
}
