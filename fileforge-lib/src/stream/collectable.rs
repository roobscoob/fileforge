use core::future::Future;

use super::ReadableStream;

pub trait Collectable<const NODE_NAME_SIZE: usize, S: ReadableStream<NODE_NAME_SIZE>> {
  type Error;

  fn collect(&mut self, stream: &mut S) -> impl Future<Output = Result<(), Self::Error>>;
}
