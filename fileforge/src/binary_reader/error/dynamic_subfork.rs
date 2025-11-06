use crate::{
  diagnostic::pool::DiagnosticPoolProvider,
  error::{FileforgeError, report::Report},
  stream::DynamicPartitionableStream,
};

use super::seek_out_of_bounds::SeekOutOfBounds;

pub enum DynamicSubforkError<'l, 'pool, S: DynamicPartitionableStream<'l>> {
  Stream(S::PartitionError),
  OutOfBounds(SeekOutOfBounds<'pool>),
}

impl<'l, 'pool, S: DynamicPartitionableStream<'l>> FileforgeError for DynamicSubforkError<'l, 'pool, S> {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider>(
    &self,
    provider: &'pool_ref P,
    callback: impl for<'tag, 'b, 'p2> FnMut(Report<'tag, 'b, 'p2, 'pool_ref, ITEM_NAME_SIZE, P>) -> (),
  ) {
    match self {
      Self::Stream(s) => s.render_into_report(provider, callback),
      Self::OutOfBounds(oob) => oob.render_into_report(provider, callback),
    }
  }
}
