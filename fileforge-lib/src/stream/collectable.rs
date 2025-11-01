use super::ReadableStream;

pub trait Collectable<S: ReadableStream> {
  type Error;

  async fn collect(&mut self, stream: &mut S) -> Result<(), Self::Error>;
}
