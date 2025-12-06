use crate::stream::ReadableStream;

pub struct MappedStream<S: ReadableStream, R, Mapper: AsyncFn(S::Type) -> R> {
  pub(super) stream: S,
  pub(super) mapper: Mapper,
}
