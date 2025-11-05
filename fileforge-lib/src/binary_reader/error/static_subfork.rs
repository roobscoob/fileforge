use crate::{diagnostic::pool::DiagnosticPoolProvider, error::FileforgeError, stream::StaticPartitionableStream};

use super::seek_out_of_bounds::SeekOutOfBounds;

pub enum StaticSubforkError<'pool, const SIZE: usize, S: StaticPartitionableStream<SIZE>> {
  Stream(S::PartitionError),
  OutOfBounds(SeekOutOfBounds<'pool>),
}

impl<'pool, const SIZE: usize, S: StaticPartitionableStream<SIZE>> FileforgeError for StaticSubforkError<'pool, SIZE, S> {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider>(
    &self,
    provider: &'pool_ref P,
    callback: impl for<'tag, 'b, 'p2> FnMut(crate::error::report::Report<'tag, 'b, 'p2, 'pool_ref, ITEM_NAME_SIZE, P>) -> (),
  ) {
    match self {
      Self::Stream(s) => s.render_into_report(provider, callback),
      Self::OutOfBounds(oob) => oob.render_into_report(provider, callback),
    }
  }
}
