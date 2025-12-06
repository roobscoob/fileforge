use crate::stream::ReadableStream;

pub struct FilteredStream<S: ReadableStream, Filter: for<'a> AsyncFn(&'a S::Type) -> bool> {
  pub(super) stream: S,
  pub(super) filter: Filter,
}
