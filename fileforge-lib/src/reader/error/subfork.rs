use crate::{error::FileforgeError, stream::DynamicPartitionableStream};

use super::seek_out_of_bounds::SeekOutOfBounds;

pub enum SubforkError<'l, 'pool, const NODE_NAME_SIZE: usize, S: DynamicPartitionableStream<'l, NODE_NAME_SIZE>> {
  Stream(S::PartitionError),
  OutOfBounds(SeekOutOfBounds<'pool, NODE_NAME_SIZE>),
}

impl<'l, 'pool, const NODE_NAME_SIZE: usize, S: DynamicPartitionableStream<'l, NODE_NAME_SIZE>> FileforgeError<NODE_NAME_SIZE> for SubforkError<'l, 'pool, NODE_NAME_SIZE, S> {
  fn render_into_report(&self, callback: impl FnMut(crate::error::report::Report<NODE_NAME_SIZE>) -> ()) {
    match self {
      Self::Stream(s) => s.render_into_report(callback),
      Self::OutOfBounds(oob) => oob.render_into_report(callback),
    }
  }
}
